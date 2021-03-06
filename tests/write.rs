use hdf5::{Data, File, IntoData, Writer};
use temporary::Directory;

macro_rules! test(
    ($($name:ident := $value:expr,)*) => ({
        let directory = Directory::new("hdf5").unwrap();
        let file = File::new(directory.join("data.h5")).unwrap();
        $({
            let value = $value;
            let value = value.into_data().unwrap();
            let dimensions = value.dimensions();
            let mut writer = Writer::new(&file, stringify!($name), dimensions);
            writer.write(&value, &vec![0; dimensions.len()], dimensions).unwrap();
        })*
    });
);

#[test]
fn boolean() {
    test!(
        a := true,
        b := false,
    );
}

#[test]
fn numeric_scalar() {
    test!(
        a := 42f32,
        b := 42f64,

        c := 42i8,
        d := 42u8,

        e := 42i16,
        f := 42u16,

        g := 42i32,
        h := 42u32,

        i := 42i64,
        j := 42u64,

        k := 42isize,
        l := 42usize,
    );
}

#[test]
fn numeric_vector() {
    test!(
        a := &vec![42f32, 69f32],
        b := &vec![42f64, 69f64],

        c := &vec![42i8, 69i8],
        d := &vec![42u8, 69u8],

        e := &vec![42i16, 69i16],
        f := &vec![42u16, 69u16],

        g := &vec![42i32, 69i32],
        h := &vec![42u32, 69u32],

        i := &vec![42i64, 69i64],
        j := &vec![42u64, 69u64],

        k := &vec![42isize, 69isize],
        l := &vec![42usize, 69usize],
    );
}

#[test]
fn overwrite() {
    test!(
        a := 42f32,
        a := 42f64,
    );
}

#[test]
fn patch() {
    let directory = Directory::new("hdf5").unwrap();
    let file = File::new(directory.join("data.h5")).unwrap();

    let mut writer = Writer::new(&file, "foo", &[10, 10]);

    writer.write(&vec![0u8; 10 * 10], &[0, 0], &[10, 10]).unwrap();
    writer.write(42u8, &[4, 2], &[1, 1]).unwrap();
    writer.write(69u8, &[6, 9], &[1, 1]).unwrap();
}

#[test]
fn reopen() {
    let directory = Directory::new("hdf5").unwrap();
    {
        let file = File::new(directory.join("data.h5")).unwrap();
        file.write("a", 42).unwrap();
    }
    {
        let file = File::new(directory.join("data.h5")).unwrap();
        file.write("a", 42).unwrap();
    }
    {
        let file = File::open(directory.join("data.h5")).unwrap();
        file.write("a", 42).unwrap();
    }
}

#[test]
fn stress() {
    use std::sync::Arc;
    use std::thread;

    let directory = Directory::new("hdf5").unwrap();
    let file = Arc::new(File::new(directory.join("data.h5")).unwrap());

    let guards = (0..100).map(|i| {
        let file = file.clone();
        thread::spawn(move || {
            let mut writer = Writer::new(&*file, &format!("number{}", i), &[1]);
            writer.write(i, &[0], &[1]).unwrap();
            true
        })
    }).collect::<Vec<_>>();

    for guard in guards {
        assert!(guard.join().unwrap());
    }
}

#[test]
fn text() {
    test!(
        a := '界',
        b := "Hello, 世界!",
    );
}
