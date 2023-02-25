//! ### BitVector Module
//!
//! BitVector uses one bit to represent a bool state.
//! BitVector is useful for the programs that need fast set operation (intersection, union,
//! difference), because that all these operations can be done with simple bitand, bitor, bitxor.
//!
//! Usually, the length of a BitVector should not be changed after constructed, for example:
//!
//! ```
//! extern crate bitvector;
//! use bitvector::*;
//!
//! fn main(){
//!   // a bitvector contains 30 elements
//!   let mut bitvec = BitVector::new(30);
//!   // add 10 elements
//!   for i in 0 .. 10 { bitvec.insert(i); }
//!   // you can use Iterator to iter over all the elements
//!   assert_eq!(bitvec.iter().collect::<Vec<_>>(), vec![0,1,2,3,4,5,6,7,8,9]);
//!
//!   let mut bitvec2 = BitVector::new(30);
//!   for i in 5 .. 15 { bitvec2.insert(i); }
//!
//!   // set union
//!   assert_eq!(bitvec.union(&bitvec2).iter().collect::<Vec<_>>(),
//!              vec![0,1,2,3,4,5,6,7,8,9,10,11,12,13,14]);
//!
//!   // set intersection
//!   assert_eq!(bitvec.intersection(&bitvec2).iter().collect::<Vec<_>>(),
//!              vec![5,6,7,8,9]);
//!
//!   // set difference
//!   assert_eq!(bitvec.difference(&bitvec2).iter().collect::<Vec<_>>(),
//!              vec![0,1,2,3,4]);
//!
//!   // you can also use `&`(intersection) `|`(union) and `^`(difference)
//!   // to do the set operations
//!   assert_eq!((&bitvec ^ &bitvec2).iter().collect::<Vec<_>>(),
//!              vec![0,1,2,3,4]);
//! }
//! ```
//!
//! ### Implementation Details
//!
//! BitVector is realized with a `Vec<u64>`. Each bit of an u64 represent if a elements exists.
//! BitVector always increases from the end to begin, it meats that if you add element `0` to an
//! empty bitvector, then the `Vec<u64>` will change from `0x00` to `0x01`.
//!
//! Of course, if the real length of set can not be divided by 64,
//! it will have a `capacity() % 64` bit memory waste.
//!

#![cfg_attr(feature = "unstable", feature(test))]
use std::fmt;
use std::iter::FromIterator;
use std::ops::*;

/// Bitvector
#[derive(Clone)]
pub struct BitVector {
    vector: Vec<u64>,
}

impl fmt::Display for BitVector {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[")?;
        for x in self.iter() {
            write!(f, "{}, ", x)?;
        }
        write!(f, "]")
    }
}

impl fmt::Debug for BitVector {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[")?;
        for x in self.vector.iter() {
            write!(f, "{:b} ", x)?;
        }
        write!(f, "]")
    }
}

impl PartialEq for BitVector {
    fn eq(&self, other: &BitVector) -> bool {
        self.eq_left(other, self.capacity())
    }
}

impl BitVector {
    /// Build a new empty bitvector
    pub fn new(bits: usize) -> Self {
        BitVector {
            vector: vec![0; u64s(bits)],
        }
    }

    /// new bitvector contains all elements
    ///
    /// If `bits % 64 > 0`, the last u64 is guaranteed not to
    /// have any extra 1 bits.
    pub fn ones(bits: usize) -> Self {
        let (word, offset) = word_offset(bits);
        let mut bvec = vec![u64::max_value(); word];
        bvec.push(u64::max_value() >> (64 - offset));
        BitVector { vector: bvec }
    }

    /// return if this set is empty
    ///
    /// if set does not contain any elements, return true;
    /// else return false.
    ///
    /// This method is averagely faster than `self.len() > 0`.
    pub fn is_empty(&self) -> bool {
        self.vector.iter().all(|&x| x == 0)
    }

    /// the number of elements in set
    pub fn len(&self) -> usize {
        self.vector
            .iter()
            .fold(0usize, |x0, x| x0 + x.count_ones() as usize)
    }

    /// Clear all elements from a bitvector
    pub fn clear(&mut self) {
        for p in &mut self.vector {
            *p = 0;
        }
    }

    pub fn filter_in_place(&mut self, mut filter: impl FnMut(usize) -> bool) {
        // println!("TODO: fix unecessary alloc");
        let len = self.capacity();
        let mut new_rem = BitVector::new(len);
        for i in 0..len {
            if self.contains(i) && filter(i) {
                new_rem.insert(i);
            }
        }
        self.intersection_inplace(&new_rem);

        // for (v, v2) in self.iter() {
        //     if *v != 0 {
        //         *v &= *v2;
        //     }
        // }
    }

    /// If `bit` belongs to set, return `true`, else return `false`.
    ///
    /// Insert, remove and contains do not do bound check.
    pub fn contains(&self, bit: usize) -> bool {
        if bit >= self.capacity() {
            return false;
        }
        let (word, mask) = word_mask(bit);
        (self.vector[word] & mask) != 0
    }

    /// compare if the following is true:
    ///
    /// self \cap {0, 1, ... , bit - 1} == other \cap {0, 1, ... ,bit - 1}
    ///
    /// for example:
    ///
    /// ```
    /// use bitvector::*;
    ///
    /// let mut A = BitVector::new(11);
    /// let mut B = BitVector::new(11);
    /// for i in vec![0, 1, 3 ,5 ,7, 10] { A.insert(i); }
    /// for i in vec![0, 1, 3, 4, 5, 7, 10] { B.insert(i); }
    ///
    ///
    /// assert!(A.eq_left(&B, 1));  // [0             ]  = [0              ]
    /// assert!(A.eq_left(&B, 2));  // [0, 1          ]  = [0, 1           ]
    /// assert!(A.eq_left(&B, 3));  // [0, 1          ]  = [0, 1           ]
    /// assert!(A.eq_left(&B, 4));  // [0, 1,   3     ]  = [0, 1,   3      ]
    /// assert!(!A.eq_left(&B, 5)); // [0, 1,   3     ] != [0, 1,   3, 4   ]
    /// assert!(!A.eq_left(&B, 6)); // [0, 1,   3,   5] != [0, 1,   3, 4, 5]
    /// ```
    ///
    pub fn eq_left(&self, other: &BitVector, bit: usize) -> bool {
        if bit == 0 {
            return true;
        }
        let (word, offset) = word_offset(bit - 1);
        // We can also use slice comparison, which only take 1 line.
        // However, it has been reported that the `Eq` implementation of slice
        // is extremly slow.
        //
        // self.vector.as_slice()[0 .. word] == other.vector.as_slice[0 .. word]
        //
        self.vector
            .iter()
            .zip(other.vector.iter())
            .take(word)
            .all(|(s1, s2)| s1 == s2)
            && (self.vector[word] << (63 - offset)) == (other.vector[word] << (63 - offset))
    }

    /// insert a new element to set
    ///
    /// If value is inserted, return true,
    /// if value already exists in set, return false.
    ///
    /// Insert, remove and contains do not do bound check.
    pub fn insert(&mut self, bit: usize) -> bool {
        if bit >= self.capacity() {
            self.grow(bit + 1);
        }
        let (word, mask) = word_mask(bit);
        let data = &mut self.vector[word];
        let value = *data;
        let new_value = value | mask;
        *data = new_value;
        new_value != value
    }

    /// remove an element from set
    ///
    /// If value is removed, return true,
    /// if value doesn't exist in set, return false.
    ///
    /// Insert, remove and contains do not do bound check.
    pub fn remove(&mut self, bit: usize) -> bool {
        let (word, mask) = word_mask(bit);
        let data = &mut self.vector[word];
        let value = *data;
        let new_value = value & !mask;
        *data = new_value;
        new_value != value
    }

    /// import elements from another bitvector
    ///
    /// If any new value is inserted, return true,
    /// else return false.
    pub fn insert_all(&mut self, all: &BitVector) -> bool {
        assert!(self.vector.len() == all.vector.len());
        let mut changed = false;
        for (i, j) in self.vector.iter_mut().zip(&all.vector) {
            let value = *i;
            *i = value | *j;
            if value != *i {
                changed = true;
            }
        }
        changed
    }

    /// the max number of elements can be inserted into set
    pub fn capacity(&self) -> usize {
        self.vector.len() * std::mem::size_of::<u64>() * 8
    }

    /// set union
    pub fn union(&self, other: &BitVector) -> BitVector {
        let v = self
            .vector
            .iter()
            .zip(other.vector.iter())
            .map(|(x1, x2)| x1 | x2);
        let len1 = self.vector.len();
        let len2 = other.vector.len();
        if len1 > len2 {
            BitVector {
                vector: v.chain(self.vector.iter().skip(len2).cloned()).collect(),
            }
        } else if len1 < len2 {
            BitVector {
                vector: v.chain(other.vector.iter().skip(len1).cloned()).collect(),
            }
        } else {
            BitVector {
                vector: v.collect(),
            }
        }
    }

    /// set intersection
    pub fn intersection(&self, other: &BitVector) -> BitVector {
        BitVector {
            vector: self
                .vector
                .iter()
                .zip(other.vector.iter())
                .map(|(x1, x2)| if *x1 == 0 { 0 } else { x1 & x2 })
                .collect(),
        }
    }

    /// set difference
    pub fn difference(&self, other: &BitVector) -> BitVector {
        let v = self.vector.iter().zip(other.vector.iter()).map(|(x1, x2)| {
            if *x1 == 0 {
                0
            } else {
                (x1 ^ x2) & x1
            }
        });
        let len1 = self.vector.len();
        let len2 = other.vector.len();
        if len1 > len2 {
            BitVector {
                vector: v.chain(self.vector.iter().skip(len2).cloned()).collect(),
            }
        } else {
            BitVector {
                vector: v.collect(),
            }
        }
    }

    pub fn difference_d(&self, other: &BitVector) -> BitVector {
        let v = self
            .vector
            .iter()
            .zip(other.vector.iter())
            .map(|(x1, x2)| x1 ^ x2);
        let len1 = self.vector.len();
        let len2 = other.vector.len();
        if len1 > len2 {
            BitVector {
                vector: v.chain(self.vector.iter().skip(len2).cloned()).collect(),
            }
        } else if len1 < len2 {
            BitVector {
                vector: v.chain(other.vector.iter().skip(len1).cloned()).collect(),
            }
        } else {
            BitVector {
                vector: v.collect(),
            }
        }
    }

    /// Union operator by modifying `self`
    ///
    /// No extra memory allocation
    pub fn union_inplace(&mut self, other: &BitVector) -> &mut BitVector {
        assert_eq!(self.capacity(), other.capacity());
        for (v, v2) in self.vector.iter_mut().zip(other.vector.iter()) {
            if *v != u64::max_value() {
                *v |= *v2;
            }
        }
        self
    }

    /// Intersection operator by modifying `self`
    ///
    /// No extra memory allocation
    pub fn intersection_inplace(&mut self, other: &BitVector) -> &mut BitVector {
        assert_eq!(self.capacity(), other.capacity());
        for (v, v2) in self.vector.iter_mut().zip(other.vector.iter()) {
            if *v != 0 {
                *v &= *v2;
            }
        }
        self
    }

    /// Difference operator by modifying `self`
    ///
    /// No extra memory allocation
    pub fn difference_inplace(&mut self, other: &BitVector) -> &mut BitVector {
        assert_eq!(self.capacity(), other.capacity());
        for (v, v2) in self.vector.iter_mut().zip(other.vector.iter()) {
            if *v != 0 {
                *v &= *v ^ *v2
            }
        }
        self
    }

    pub fn difference_d_inplace(&mut self, other: &BitVector) -> &mut BitVector {
        assert_eq!(self.capacity(), other.capacity());
        for (v, v2) in self.vector.iter_mut().zip(other.vector.iter()) {
            *v ^= *v2;
        }
        self
    }

    fn grow(&mut self, num_bits: usize) {
        let num_words = u64s(num_bits);
        if self.vector.len() < num_words {
            self.vector.resize(num_words, 0)
        }
    }

    /// Return a iterator of element based on current bitvector,
    /// for example:
    ///
    /// ```
    /// extern crate bitvector;
    /// use bitvector::*;
    ///
    /// fn main() {
    ///     let mut bitvec = BitVector::new(5);
    ///     bitvec.insert(2);
    ///     bitvec.insert(3);
    ///     // The bitvector becomes: 0x00 0x00 0x00 0x0C
    ///     assert_eq!(bitvec.iter().collect::<Vec<_>>(), vec![2,3]);
    ///     // collected vector will contains the real element not the bit.
    /// }
    /// ```
    pub fn iter<'a>(&'a self) -> BitVectorIter<'a> {
        BitVectorIter {
            iter: self.vector.iter(),
            current: 0,
            idx: 0,
            size: self.capacity(),
        }
    }
}

impl std::iter::IntoIterator for BitVector {
    type Item = usize;
    type IntoIter = BitVectorIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        BitVectorIntoIter {
            size: self.capacity(),
            content: self.vector,
            slice_index: 0,
            current: 0,
            idx: 0,
        }
    }
}

impl<'a> std::iter::IntoIterator for &'a BitVector {
    type Item = usize;
    type IntoIter = BitVectorIter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        BitVectorIter {
            iter: self.vector.iter(),
            current: 0,
            idx: 0,
            size: self.capacity(),
        }
    }
}

impl<'a> std::iter::IntoIterator for &'a mut BitVector {
    type Item = usize;
    type IntoIter = BitVectorIter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        BitVectorIter {
            iter: self.vector.iter(),
            current: 0,
            idx: 0,
            size: self.capacity(),
        }
    }
}

pub struct BitVectorIntoIter {
    content: Vec<u64>,
    slice_index: usize,
    current: u64,
    idx: usize,
    size: usize,
}

impl Iterator for BitVectorIntoIter {
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        if self.idx > self.size {
            return None;
        }
        while self.current == 0 {
            self.current = if let Some(&i) = self.content.get(self.slice_index) {
                self.slice_index += 1;
                if i == 0 {
                    self.idx += 64;
                    continue;
                } else {
                    self.idx = u64s(self.idx) * 64;
                    i
                }
            } else {
                return None;
            }
        }
        let offset = self.current.trailing_zeros() as usize;
        self.current >>= offset;
        self.current >>= 1; // shift otherwise overflows for 0b1000_0000_…_0000
        self.idx += offset + 1;
        return Some(self.idx - 1);
    }
}

/// Iterator for BitVector
pub struct BitVectorIter<'a> {
    iter: ::std::slice::Iter<'a, u64>,
    current: u64,
    idx: usize,
    size: usize,
}

impl<'a> Iterator for BitVectorIter<'a> {
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        if self.idx > self.size {
            return None;
        }
        while self.current == 0 {
            self.current = if let Some(&i) = self.iter.next() {
                if i == 0 {
                    self.idx += 64;
                    continue;
                } else {
                    self.idx = u64s(self.idx) * 64;
                    i
                }
            } else {
                return None;
            }
        }
        let offset = self.current.trailing_zeros() as usize;
        self.current >>= offset;
        self.current >>= 1; // shift otherwise overflows for 0b1000_0000_…_0000
        self.idx += offset + 1;
        return Some(self.idx - 1);
    }
}

macro_rules! impl_from_iterator {
    ($t: tt) => {
        impl FromIterator<$t> for BitVector {
            fn from_iter<I>(iter: I) -> BitVector
            where
                I: IntoIterator<Item = $t>,
            {
                let iter = iter.into_iter();
                let mut bv = BitVector::new(2 << 12);
                for val in iter {
                    bv.insert(val as usize);
                }
                bv
            }
        }
    };
}

impl_from_iterator!(u8);
impl_from_iterator!(u16);
impl_from_iterator!(u32);
impl_from_iterator!(u64);
impl_from_iterator!(usize);
impl_from_iterator!(i8);
impl_from_iterator!(i16);
impl_from_iterator!(i32);
impl_from_iterator!(i64);
impl_from_iterator!(isize);

impl FromIterator<bool> for BitVector {
    fn from_iter<I>(iter: I) -> BitVector
    where
        I: IntoIterator<Item = bool>,
    {
        let iter = iter.into_iter();
        let (len, _) = iter.size_hint();
        // Make the minimum length for the bitvector 64 bits since that's
        // the smallest non-zero size anyway.
        let len = if len < 64 { 64 } else { len };
        let mut bv = BitVector::new(len);
        for (idx, val) in iter.enumerate() {
            if val {
                bv.insert(idx);
            }
        }

        bv
    }
}

impl<'a> BitAnd for &'a BitVector {
    type Output = BitVector;
    fn bitand(self, rhs: Self) -> BitVector {
        self.intersection(rhs)
    }
}

impl<'a> BitAndAssign for &'a mut BitVector {
    fn bitand_assign(&mut self, rhs: Self) {
        self.intersection_inplace(rhs);
    }
}

impl<'a> BitOr for &'a BitVector {
    type Output = BitVector;
    fn bitor(self, rhs: Self) -> BitVector {
        self.union(rhs)
    }
}

impl<'a> BitOrAssign for &'a mut BitVector {
    fn bitor_assign(&mut self, rhs: Self) {
        self.union_inplace(rhs);
    }
}

impl<'a> BitXor for &'a BitVector {
    type Output = BitVector;
    fn bitxor(self, rhs: Self) -> BitVector {
        self.difference(rhs)
    }
}

impl<'a> BitXorAssign for &'a mut BitVector {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.difference_inplace(rhs);
    }
}

impl BitAnd for BitVector {
    type Output = BitVector;
    fn bitand(self, rhs: Self) -> BitVector {
        self.intersection(&rhs)
    }
}

impl BitAndAssign for BitVector {
    fn bitand_assign(&mut self, rhs: Self) {
        self.intersection_inplace(&rhs);
    }
}

impl BitOr for BitVector {
    type Output = BitVector;
    fn bitor(self, rhs: Self) -> BitVector {
        self.union(&rhs)
    }
}

impl BitOrAssign for BitVector {
    fn bitor_assign(&mut self, rhs: Self) {
        self.union_inplace(&rhs);
    }
}

impl BitXor for BitVector {
    type Output = BitVector;
    fn bitxor(self, rhs: Self) -> BitVector {
        self.difference(&rhs)
    }
}

impl BitXorAssign for BitVector {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.difference_inplace(&rhs);
    }
}

fn u64s(elements: usize) -> usize {
    (elements + 63) / 64
}

fn word_offset(index: usize) -> (usize, usize) {
    (index / 64, index % 64)
}

fn word_mask(index: usize) -> (usize, u64) {
    let word = index / 64;
    let mask = 1 << (index % 64);
    (word, mask)
}
