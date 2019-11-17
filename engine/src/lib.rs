extern crate claxon;
pub extern crate cpal;
pub extern crate vst;
pub extern crate sample;
#[macro_use]
extern crate log;
#[macro_use]
extern crate failure;
extern crate generational_arena;

pub mod devices;
pub mod loader;
pub mod prelude;
pub mod supervisor;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
