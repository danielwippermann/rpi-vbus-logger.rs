use resol_vbus::{Data, DataSet};

pub enum DataSetStabilityState {
    DataSetChanged,
    Stabilizing(i32),
    Stabilized,
    Stable,
}

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

    pub fn data_set(&self) -> &DataSet {
        &self.data_set
    }

    pub fn as_data_slice(&self) -> &[Data] {
        self.data_set.as_data_slice()
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

    pub fn add_data(&mut self, other: Data) -> DataSetStabilityState {
        let old_len = self.data_set.len();

        self.data_set.add_data(other);

        self.handle_added_data(old_len)
    }

    // pub fn add_data_set(&mut self, other: DataSet) -> DataSetStabilityState {
    //     let old_len = self.data_set.len();

    //     self.data_set.add_data_set(other);

    //     self.handle_added_data(old_len)
    // }

    fn handle_added_data(&mut self, old_len: usize) -> DataSetStabilityState {
        if self.data_set.len() != old_len {
            self.counter = 0;
            self.is_stable = false;
            DataSetStabilityState::DataSetChanged
        } else if self.is_stable {
            DataSetStabilityState::Stable
        } else {
            self.counter += 1;
            let percent = self.stability_percent();
            if percent >= 100 {
                self.is_stable = true;
                DataSetStabilityState::Stabilized
            } else {
                DataSetStabilityState::Stabilizing(percent)
            }
        }
    }
}
