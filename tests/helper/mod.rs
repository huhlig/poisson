#![allow(unused)]
use poisson::{PoissonDisk, VecLike};

use rand::{SeedableRng, XorShiftRng};

use std::fmt::Debug;

use na::Norm;


pub fn test_with_samples<T>(samples: u32, relative_radius: f64, seeds: u32, periodicity: bool)
    where T: Debug + VecLike
{
    test_with_samples_prefilled(samples, relative_radius, seeds, periodicity, None::<T>);
}

pub fn test_with_samples_prefilled<T, I>(samples: u32, relative_radius: f64, seeds: u32, periodicity: bool, prefiller: I)
    where T: Debug + VecLike, I: IntoIterator<Item=T> + Clone
{
    for i in 0..seeds {
        let mut prefilled = vec![];
        let rand = XorShiftRng::from_seed([i + 1, seeds - i + 1, (i + 1) * (i + 1), 1]);
        let mut poisson = PoissonDisk::new(rand);
        if periodicity {
            poisson = poisson.perioditic();
        }
        let mut poisson_iter = poisson.build_samples(samples, relative_radius).into_iter();
        for s in prefiller.clone().into_iter() {
            prefilled.push(s);
            poisson_iter.insert(s);
        }
        let radius = poisson_iter.radius();
        let periodicity = poisson_iter.periodicity();
        test_poisson(poisson_iter.chain(prefilled.into_iter()), radius, periodicity);
    }
}

pub fn test_poisson<I, T>(poisson: I, radius: f64, periodicity: bool)
    where I: Iterator<Item=T>, T: Debug + VecLike
{
    let dim = T::dim(None);
    let mut vecs = vec![];
    let mut hints = vec![];
    {
        let mut iter = poisson.into_iter();
        while let Some(v) = iter.next() {
            if let (low, Some(high)) = iter.size_hint() {
                hints.push((low, high));
            } else {
                panic!("There wasn't hint for {}th iteration.", hints.len());
            }
            vecs.push(v);
        }
    }
    let len = hints.len();
    for (n, (l, h)) in hints.into_iter().enumerate() {
        let remaining = len - (n + 1);
        assert!(l <= remaining, "Lower bound of hint should be smaller than or equal to actual: {} <= {}", l, remaining);
        assert!(h >= remaining, "Upper bound of hint should be larger than or equal to actual: {} >= {}", h, remaining);
    }
    //TODO: Figure out how to check if distribution is maximal.
    // let packing_density = vecs.len() as f64 * ::sphere::sphere_volume(poisson.radius(), dim as u64);
    // println!("{}", packing_density);
    // panic!();
    let vecs = if periodicity {
        let mut vecs2 = vec![];
        for n in 0..3i64.pow(dim as u32) {
            let mut t = T::zero();
            let mut div = n;
            for i in 0..dim {
                let rem = div % 3;
                div /= 3;
                t[i] = (rem - 1) as f64;
            }
            for v in &vecs {
                vecs2.push(*v + t);
            }
        }
        vecs2
    } else {
        vecs
    };
    assert_legal_poisson(&vecs, radius);
}

pub fn assert_legal_poisson<T>(vecs: &Vec<T>, radius: f64)
    where T: Debug + VecLike
{
    for &v1 in vecs {
        for &v2 in vecs {
            if v1 == v2 {
                continue;
            }
            let dist = (v1 - v2).norm();
            assert!(dist >= radius * 2.,
                    "Poisson-disk distribution requirement not met: There exists 2 vectors with \
                     distance to each other of {} which is smaller than smallest allowed one {}. \
                     The samples: [{:?}, {:?}]",
                    dist,
                    radius * 2.,
                    v1,
                    v2);
        }
    }
}
