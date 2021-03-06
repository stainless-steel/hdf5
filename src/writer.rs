use data::{Data, IntoData};
use dataset::{self, Dataset};
use dataspace;
use datatype::Datatype;
use file::File;
use link::Link;
use {Location, Result};

/// A writer.
///
/// Writers are suitable for storing large arrays.
pub struct Writer<'l> {
    state: State<'l>,
}

enum State<'l> {
    Setup { location: &'l File, name: String, dimensions: Vec<usize> },
    Ready(Inner),
}

struct Inner {
    dataset: Dataset,
    datatype: Datatype,
    dimensions: usize,
}

impl<'l> Writer<'l> {
    /// Create a writer.
    ///
    /// If there exists a dataset with the same name, it will be removed from
    /// the file structure, and a new dataset will be created. This operation,
    /// however, does not reclaim the corresponding space. See [Section
    /// 5.5.2][1] in HDF5 User’s Guide for further details.
    ///
    /// [1]: https://www.hdfgroup.org/HDF5/doc/UG/10_Datasets.html#Allocation
    pub fn new(file: &'l File, name: &str, dimensions: &[usize]) -> Writer<'l> {
        Writer {
            state: State::Setup {
                location: file,
                name: name.to_string(),
                dimensions: dimensions.to_vec(),
            },
        }
    }

    /// Write data.
    ///
    /// The function writes a chunk of data at a particular position with a
    /// particular size. The datatype should stay unchanged from one invocation
    /// to another.
    pub fn write<T: IntoData>(&mut self, data: T, position: &[usize], size: &[usize])
                              -> Result<()> {

        let data = try!(data.into_data());
        let state = match self.state {
            State::Ready(ref mut inner) => return inner.write(data, position, size),
            State::Setup { location, ref name, ref dimensions } => {
                State::Ready(try!(Inner::new(location, name, data.datatype(), dimensions)))
            },
        };
        self.state = state;
        self.write(data, position, size)
    }
}

impl Inner {
    fn new<T: Location>(location: T, name: &str, datatype: Datatype, dimensions: &[usize])
                        -> Result<Inner> {

        if try!(Link::exists(&location, name)) {
            try!(Link::delete(&location, name));
        }
        let dataspace = try!(dataspace::new(dimensions));
        let dataset = try!(dataset::new(&location, name, &datatype, &dataspace));
        Ok(Inner { dataset: dataset, datatype: datatype, dimensions: dimensions.len() })
    }

    fn write<T: Data>(&mut self, data: T, position: &[usize], size: &[usize]) -> Result<()> {
        if self.datatype != data.datatype() {
            raise!("the data should have the claimed datatype");
        }
        if self.dimensions != position.len() {
            raise!("the position should have the claimed number of dimensions");
        }
        if self.dimensions != size.len() {
            raise!("the size should have the claimed number of dimensions");
        }
        if product!(data.dimensions()) != product!(size) {
            raise!("the data should have the claimed number of elements");
        }

        let memory_space = try!(dataspace::new(size));
        let file_space = try!(self.dataset.space());
        try!(file_space.select(position, size));

        self.dataset.write(data, &memory_space, &file_space)
    }
}
