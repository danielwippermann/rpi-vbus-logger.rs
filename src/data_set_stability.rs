use resol_vbus::DataSet;


pub struct DataSetStability {
    data_set: DataSet,
    counter: usize,
    is_stable: bool,
}


impl DataSetStability {

    pub fn new() -> DataSetStability {
        DataSetStability {
            data_set: DataSet::new(),
            counter: 0,
            is_stable: false,
        }
    }

    pub fn is_stable(&self) -> bool {
        self.is_stable
    }

    pub fn stability_percent(&self) -> i32 {
        let count = self.data_set.len() * 3;
        if self.is_stable {
            100
        } else if count > 0 {
            (self.counter * 100 / count) as i32
        } else {
            0
        }
    }

    pub fn add_data_set(&mut self, other: DataSet) {
        let old_len = self.data_set.len();

        self.data_set.add_data_set(other);

        if self.data_set.len() != old_len {
            self.counter = 0;
            self.is_stable = false;
        } else if self.is_stable {
            // nop
        } else if self.stability_percent() >= 100 {
            self.is_stable = true;
        }
    }

}
