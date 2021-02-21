/// The module which holds extensions for the slice type ([T])

use std::ops::Index;

pub struct ExtractResultMut<'a, T : 'a> {
    left : &'a mut[T],
    right : &'a mut[T],
}

// `SliceIndex` is unstable so we impl just for `usize`
impl<'a, T : 'a> Index<usize> for ExtractResultMut<'a, T> {
    type Output = T;

    fn index(&self, id : usize) -> &Self::Output {
        assert_ne!(id, self.left.len(), "Can't access the middle element in the extractor");

        if id < self.left.len() {
            &self.left[id]
        } else {
            &self.right[id - self.left.len() - 1]
        }
    }
}

pub trait SliceExt {
    type Elem;

    fn extract_mut(&mut self, mid : usize) -> (ExtractResultMut<Self::Elem>, &mut Self::Elem);
}

impl<T> SliceExt for [T] {
    type Elem = T;

    #[inline]
    fn extract_mut(&mut self, mid : usize) -> (ExtractResultMut<Self::Elem>, &mut Self::Elem) {
        let (left, mid_right) = self.split_at_mut(mid);
        let (mid, right) = mid_right.split_first_mut().unwrap();

        (
            ExtractResultMut { left, right },
            mid
        )
    }
}

#[cfg(test)]
mod tests {
    use super::SliceExt;

    #[test]
    fn test_extract_correct1() {
        let mut arr = [1, 5, 6, 7, 1, 5];

        let (sides, mid) = arr.extract_mut(3);

        assert_eq!(*mid, 7);
        assert_eq!(sides.left, [1, 5, 6].as_ref());
        assert_eq!(sides.right, [1, 5].as_ref());
        assert_eq!(sides.left.len() + 1 + sides.right.len(), arr.len());
    }
    
    #[test]
    fn test_extract_correct2() {
        let mut arr = [1, 5, 6, 7, 1, 5];

        assert_eq!(arr[3], 7);

        {
            let (sides, mid) = arr.extract_mut(3);
            *mid = 3;
        }

        assert_eq!(arr[3], 3);
    }

    #[test]
    fn test_extract_index1() {
        let mut arr = [1, 5, 6, 7, 1, 5];

        let (sides, mid) = arr.extract_mut(3);

        assert_eq!(sides[0], 1);
        assert_eq!(sides[1], 5);
        assert_eq!(sides[2], 6);
        assert_eq!(sides[4], 1);
        assert_eq!(sides[5], 5);
    }
    
    #[test]
    #[should_panic(expected = "Can't access the middle element in the extractor")]
    fn test_extract_index2() {
        let mut arr = [1, 5, 6, 7, 1, 5];

        let (sides, mid) = arr.extract_mut(3);
    
        sides[3];
    }
}
