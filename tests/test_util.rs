use craytracer::util::partition_by;

#[cfg(test)]
pub mod partition_by {
    use crate::partition_by;

    fn check<T, F>(s: &mut [T], f: F)
    where
        T: std::fmt::Display + std::fmt::Debug,
        F: Fn(&T) -> bool,
    {
        let (left, right) = partition_by(s, &f);
        for t in left {
            assert_eq!(true, f(t), "Expected predicate to be true for {t}");
        }
        for t in right {
            assert_eq!(false, f(t), "Expected predicate to be false for {t}");
        }
    }

    #[test]
    fn empty() {
        check(&mut Vec::<usize>::new(), |&i| i > 0);
    }

    #[test]
    fn single() {
        check(&mut vec![1], |&i| i == 0);
        check(&mut vec![1], |&i| i == 1);
    }

    #[test]
    fn sorted() {
        check(&mut vec![1, 2, 3], |&i| i > 0);
        check(&mut vec![1, 2, 3], |&i| i > 1);
        check(&mut vec![1, 2, 3], |&i| i > 2);
        check(&mut vec![1, 2, 3], |&i| i > 3);
    }

    #[test]
    fn unsorted() {
        check(&mut vec![1, 2, 3, 4, 5], |&i| i % 2 == 0);
        check(&mut vec![1, 2, 3, 4, 5], |&i| i % 2 == 1);
    }
}
