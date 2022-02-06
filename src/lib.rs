//! # edt
//!
//! An implementation of 2D EDT ([Euclidian distance transform](https://en.wikipedia.org/wiki/Distance_transform)) with Saito's algorithm in pure Rust
//!
//! There are also [other](https://crates.io/crates/distance-transform)
//! [crates](https://crates.io/crates/dt) that implements EDT,
//! but I would like to reinvent a wheel that has these advantages:
//! 
//! * No dependencies (except example codes)
//! * Intuitive to use (accepts a numerical slice and a shape)
//! 
//! Performance was not the priority, but I would like to explore more optimizations.
//! 
//! EDT is the basis of many algorithms, but it is hard to find in a general purpose image processing library,
//! probably because the algorithm is not trivial to implement efficiently.
//! This crate provides an implementation of EDT in fairly efficient algorithm presented in the literature.
//!
//! The algorithm used in this crate (Saito's algorithm) is O(n^3), where n is the number of pixels along one direction.
//! Naive computation of EDT would be O(n^4), so it is certainly better than that, but there is also fast-marching based
//! algorithm that is O(n^2).
//!
//! ## Usage
//!
//! Add dependency
//!
//! ```toml
//! [dependencies]
//! edt = "0.1.0"
//! ```
//!
//! Prepare a binary image as a flattened vec.
//! This library assumes that the input is a flat vec for 2d image.
//!
//! ```rust
//! let vec: Vec<bool> = vec![/*...*/];
//! ```
//!
//! If you want to read input from an image, you can use [image](https://crates.io/crates/image) crate.
//! Make sure to put it to your project's dependencies in that case.
//!
//! ```rust
//! use image::GenericImageView;
//! let img = image::open("Rust_logo.png").unwrap();
//! let dims = img.dimensions();
//! ```
//!
//! Call edt with given shape
//!
//! ```rust
//! # let vec: Vec<bool> = vec![false; 32 * 32];
//! use edt::edt;
//!
//! let edt_image = edt(&vec, (32, 32), true);
//! ```
//!
//! Save to a file if you want.
//! The code below normalizes the value with maximum value to 8 bytes grayscale image.
//!
//! ```rust
//! # use edt::edt;
//! # let vec: Vec<bool> = vec![false; 32 * 32];
//! # let edt_image = edt(&vec, (32, 32), true);
//! use image::{ImageBuffer, Luma};
//!
//! let max_value = edt_image.iter().map(|p| *p).reduce(f64::max).unwrap();
//! let edt_img = edt_image
//!     .iter()
//!     .map(|p| (*p / max_value * 255.) as u8)
//!     .collect();
//!
//! let edt_img: ImageBuffer<Luma<u8>, Vec<u8>> =
//!     ImageBuffer::from_vec(32, 32, edt_img).unwrap();
//!
//! // Write the contents of this image to the Writer in PNG format.
//! edt_img.save("edt.png").unwrap();
//! ```
//!
//! See [examples](https://github.com/msakuta/rust-edt/tree/master/examples) folder for more.
//!
//! ## Literature
//!
//! ### 2D Euclidean Distance Transform Algorithms: A Comparative Survey
//!
//! This paper is a great summary of this field of research.
//!
//! doi=10.1.1.66.2644
//!
//! <https://citeseerx.ist.psu.edu/viewdoc/download?doi=10.1.1.66.2644&rep=rep1&type=pdf>
//!
//! Section 7.7
//!
//!
//! ### Saito and Toriwaki \[1994\] (Original paper)
//!
//! <https://www.cs.jhu.edu/~misha/ReadingSeminar/Papers/Saito94.pdf>

mod primitive_impl;

/// A trait for types that can be interpreted as a bool.
/// 
/// Primitive numerical types (integers and floats) implement this trait,
/// so you don't have to implement this by yourself.
/// However, you could implement it for your own custom type, if you want.
/// 
/// We don't use [num](https://crates.io/crates/num) crate because it is overkill for our purposes.
pub trait BoolLike {
    fn as_bool(&self) -> bool;
}

/// Produce an EDT from binary image
pub fn edt<T: BoolLike>(map: &[T], shape: (usize, usize), invert: bool) -> Vec<f64> {
    let horz_edt = horizontal_edt(map, shape, invert);

    let vertical_scan = |x, y| {
        let total_edt = (0..shape.1).map(|y2| {
            let horz_val: f64 = horz_edt[x + y2 * shape.0];
            (y2 as f64 - y as f64).powf(2.) + horz_val.powf(2.)
        });
        total_edt.reduce(f64::min).unwrap()
    };

    let mut ret = vec![0.; shape.0 * shape.1];

    for x in 0..shape.0 {
        for y in 0..shape.1 {
            ret[x + y * shape.0] = vertical_scan(x, y);
        }
    }

    ret
}

fn horizontal_edt<T: BoolLike>(map: &[T], shape: (usize, usize), invert: bool) -> Vec<f64> {
    let mut horz_edt = map
        .iter()
        .map(|b| (((b.as_bool() != invert) as usize) * map.len()) as f64)
        .collect::<Vec<f64>>();

    let scan = |x, y, min_val: &mut f64, horz_edt: &mut Vec<f64>| {
        let f: f64 = horz_edt[x + y * shape.0];
        let next = *min_val + 1.;
        let v = f.min(next);
        horz_edt[x + y * shape.0] = v;
        *min_val = v;
    };

    for y in 0..shape.1 {
        let mut min_val = 0.;
        for x in 0..shape.0 {
            scan(x, y, &mut min_val, &mut horz_edt);
        }
        min_val = 0.;
        for x in (0..shape.0).rev() {
            scan(x, y, &mut min_val, &mut horz_edt);
        }
    }

    horz_edt
}

#[cfg(test)]
mod test {
    use super::*;

    fn flatten<T>(nested: Vec<Vec<T>>) -> Vec<T> {
        nested.into_iter().flatten().collect()
    }

    fn test_map() -> Vec<bool> {
        let str_map = [
            "0000000000",
            "0001111000",
            "0011111110",
            "0001111000",
            "0000110000",
        ];
        let map = flatten(
            str_map
                .iter()
                .map(|s| s.chars().map(|c| c == '1').collect::<Vec<_>>())
                .collect::<Vec<_>>(),
        );
        map
    }

    fn reshape(v: &Vec<f64>, shape: (usize, usize)) -> Vec<Vec<f64>> {
        let mut ret = vec![];

        for y in 0..shape.1 {
            ret.push(v[y * shape.0..(y + 1) * shape.0].into());
        }

        ret
    }

    fn print_2d(v: &[Vec<f64>]) {
        for row in v {
            for cell in row {
                if *cell == 16. {
                    print!("f");
                } else {
                    print!("{:.0}", cell);
                }
            }
            print!("\n");
        }
    }

    fn parse_edt_str(s: &[&str]) -> Vec<f64> {
        flatten(
            s.iter()
                .map(|s| {
                    s.chars()
                        .map(|c| {
                            if c != 'f' {
                                (c as u8 - '0' as u8) as f64
                            } else {
                                15.
                            }
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>(),
        )
    }

    #[test]
    fn test_horizontal_edt() {
        let map = test_map();
        let str_edt = [
            "0000000000",
            "0001221000",
            "0012343210",
            "0001221000",
            "0000110000",
        ];
        print_2d(&reshape(
            &horizontal_edt(&map, (map.len() / str_edt.len(), str_edt.len()), false),
            (str_edt[0].len(), str_edt.len()),
        ));
        assert_eq!(
            horizontal_edt(&map, (map.len() / str_edt.len(), str_edt.len()), false),
            parse_edt_str(&str_edt)
        );
    }

    #[test]
    fn test_edt() {
        let map = test_map();
        let str_edt = [
            "0000000000",
            "0001111000",
            "0012442110",
            "0001221000",
            "0000110000",
        ];
        let shape = (map.len() / str_edt.len(), str_edt.len());
        let edt = edt(&map, shape, false);
        eprintln!("edt({:?}):", shape);
        print_2d(&reshape(&edt, shape));
        assert_eq!(edt, parse_edt_str(&str_edt));
    }
}
