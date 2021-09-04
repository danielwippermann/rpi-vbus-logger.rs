#![deny(warnings)]
#![deny(future_incompatible)]
#![deny(nonstandard_style)]
#![deny(rust_2018_compatibility)]
#![deny(rust_2018_idioms)]

extern crate mysql;
extern crate resol_vbus;
#[macro_use]
extern crate serde_derive;
extern crate serial;
extern crate toml;

use std::rc::Rc;
use std::time::Duration;

use mysql::prelude::Queryable;
use mysql::{OptsBuilder, Pool, PooledConn, Value};
use serial::{SerialPort, SystemPort};

use resol_vbus::chrono::{TimeZone, Utc};
use resol_vbus::*;

mod error;
use self::error::{Error, Result};

mod config;
use self::config::{read_config, DatabaseConfig, FieldConfig};

mod data_set_stability;
use self::data_set_stability::{DataSetStability, DataSetStabilityState};

struct Field<'a> {
    column: &'a str,
    stmt: String,
    packet_id: PacketId,
    packet_spec: Rc<specification::PacketSpec>,
    field_spec_pos: usize,
}

fn open_serial_port(path: &str) -> Result<SystemPort> {
    let mut port = serial::open(&path)?;

    port.reconfigure(&|settings| {
        settings.set_baud_rate(serial::Baud9600)?;
        settings.set_char_size(serial::Bits8);
        settings.set_parity(serial::ParityNone);
        settings.set_stop_bits(serial::Stop1);
        settings.set_flow_control(serial::FlowNone);
        Ok(())
    })?;

    Ok(port)
}

fn connect_to_database(config: &DatabaseConfig) -> Result<Pool> {
    let builder = OptsBuilder::new()
        .ip_or_hostname(Some(config.hostname.clone()))
        .tcp_port(config.port)
        .user(Some(config.username.clone()))
        .pass(Some(config.password.clone()))
        .db_name(Some(config.database.clone()));

    let pool = Pool::new(builder)?;

    Ok(pool)
}

fn process_fields_config<'a>(
    field_configs: &'a Vec<FieldConfig>,
    spec: &'a Specification,
    db_name: &str,
    db_conn: &mut PooledConn,
) -> Result<Vec<Field<'a>>> {
    let mut fields = Vec::with_capacity(field_configs.len());

    for fc in field_configs.iter() {
        let column_is_valid = fc.column.chars().all(|c| match c {
            '0'..='9' | 'A'..='Z' | 'a'..='z' | '_' => true,
            _ => false,
        });
        if !column_is_valid {
            return Err(Error::from(format!(
                "Invalid chars in column name {:?}",
                fc.column
            )));
        }
        match fc.column.as_str() {
            "id" | "timestamp" => {
                return Err(Error::from(format!(
                    "Reserved column name used: {:?}",
                    fc.column
                )))
            }
            _ => (),
        }

        let row_count = db_conn.exec_first("SELECT COUNT(*) FROM information_schema.COLUMNS WHERE TABLE_SCHEMA = :schema AND TABLE_NAME = :table AND COLUMN_NAME = :column", (&db_name, "data", &fc.column))
            .expect("Unable to get column information")
            .expect("Expected to get row count but got nothing");
        match row_count {
            Value::Int(1) => (),
            Value::Int(0) => {
                return Err(Error::from(format!(
                    "Unknown column name used: {:?}",
                    fc.column
                )))
            }
            _ => {
                return Err(Error::from(format!(
                    "Unexpected row_count: {:?}",
                    row_count
                )))
            }
        };

        let stmt = format!("UPDATE data SET {} = :raw_value WHERE id = :id", fc.column);
        let packet_id = fc.packet_id.to_packet_id()?;
        let packet_spec = spec.get_packet_spec_by_id(packet_id);
        let field_spec_pos = match packet_spec.get_field_spec_position(&fc.field_id) {
            Some(p) => p,
            None => {
                return Err(Error::from(format!(
                    "Unknonwn field ID {:?} for packet ID {:?}",
                    fc.field_id, fc.packet_id
                )))
            }
        };

        let field = Field {
            column: &fc.column,
            stmt: stmt,
            packet_id: packet_id,
            packet_spec: packet_spec,
            field_spec_pos: field_spec_pos,
        };

        fields.push(field);
    }

    Ok(fields)
}

fn print_fields(spec: &Specification, data_set: &DataSet) {
    let mut last_data_index = None;
    for field in spec.fields_in_data_set(data_set) {
        let current_data_index = Some(field.data_index());
        if last_data_index != current_data_index {
            last_data_index = current_data_index;
            println!(
                "- {}: {}",
                field.packet_spec().packet_id,
                field.packet_spec().name
            );
        }
        println!(
            "    - {}: {}",
            field.field_spec().field_id,
            field.field_spec().name
        );
    }
}

fn main() -> Result<()> {
    println!("Reading configuration...");
    let config = read_config().expect("Unable to read config");

    println!("Connecting to database...");
    let pool = connect_to_database(&config.database).expect("Unable to connect to database");

    let mut db_conn = pool.get_conn().expect("Unable to get database connection");

    println!("Finding last record...");
    let mut last_timestamp = match db_conn
        .exec_first("SELECT MAX(timestamp) FROM data", ())
        .expect("Unable to fetch max timestamp")
    {
        Some(row) => match row {
            Value::NULL => Utc::now(),
            Value::Date(year, month, day, hour, minute, seconds, micros) => Utc
                .ymd(year as i32, month as u32, day as u32)
                .and_hms_micro(hour as u32, minute as u32, seconds as u32, micros),
            ref other => panic!("Unsupported value {:?}", other),
        },
        None => Utc::now(),
    };

    println!("    last_timestamp = {:?}", last_timestamp);

    let spec_file = SpecificationFile::new_default();
    let spec = Specification::from_file(spec_file, Language::En);

    println!("Process field configuration...");
    let fields = process_fields_config(
        &config.fields,
        &spec,
        &config.database.database,
        &mut db_conn,
    )
    .expect("Unable to process the field config");

    println!("Connecting to VBus...");
    let mut port = open_serial_port(&config.serial.path).expect("Unable to open serial port");
    port.set_timeout(Duration::from_secs(3600))?;

    // let mut stream = std::net::TcpStream::connect("192.168.13.1:7053").unwrap();
    // let mut connector = ::resol_vbus::TcpConnector::new(stream);
    // connector.connect().unwrap();
    // let mut stream = connector.into_inner();

    let mut ldr = LiveDataReader::new(0, port);
    // let mut ldr = LiveDataReader::new(0, stream);

    let mut data_set_stability = DataSetStability::new();

    let logger_interval = config.logger.interval as i64;

    while let Some(data) = ldr.read_data().expect("Unable to read data from VBus") {
        if !data.is_packet() {
            continue;
        }

        let data_timestamp = data.as_header().timestamp;

        match data_set_stability.add_data(data) {
            DataSetStabilityState::DataSetChanged => {
                println!("Data set changed, waiting for it to stabilize again...");
            },
            DataSetStabilityState::Stabilizing(percent) => {
                println!("Data set stabilizing at {}%", percent);
            },
            DataSetStabilityState::Stabilized => {
                println!("Data set stabilized");
                print_fields(&spec, data_set_stability.data_set());
            },
            DataSetStabilityState::Stable => {
                // nop
            },
        }

        if !data_set_stability.is_stable() {
            continue;
        }

        if last_timestamp.timestamp() / logger_interval
            != data_timestamp.timestamp() / logger_interval
        {
            last_timestamp = data_timestamp;

            println!("Storing data set for timestamp {}", &last_timestamp);

            let sql_timestamp =
                mysql::chrono::NaiveDateTime::from_timestamp(last_timestamp.timestamp(), 0);

            db_conn
                .exec_drop(
                    "INSERT INTO data(timestamp) VALUES(:timestamp)",
                    (sql_timestamp,),
                )
                .expect("Unable to insert timestamp");

            let id: Value = db_conn
                .exec_first("SELECT LAST_INSERT_ID()", ())
                .expect("Unable to get insert ID")
                .expect("Expected last insert ID row but got nothing");

            for data in data_set_stability.as_data_slice() {
                let packet = data.as_packet();
                let packet_id = packet.packet_id();
                let frame_data = packet.valid_frame_data();

                // println!("   Processing packet {}", packet.id_string());

                for field in fields.iter() {
                    if field.packet_id == packet_id {
                        // println!("      Processing field {}", field.column);
                        let field_spec = field
                            .packet_spec
                            .get_field_spec_by_position(field.field_spec_pos);
                        let raw_value = field_spec.raw_value_f64(frame_data);
                        // println!("         raw_value = {:?}", raw_value);
                        db_conn
                            .exec_drop(&field.stmt, (raw_value, &id))
                            .expect(&format!("Unable to update column {}", field.column));
                    }
                }
            }
        }
    }

    Ok(())
}
