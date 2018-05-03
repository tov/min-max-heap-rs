pub trait HeapIndex {
    fn parent(self) -> Self;
    fn grandparent(self) -> Self;

    fn child1(self) -> Self;
    fn child2(self) -> Self;

    fn grandchild1(self) -> Self;
    fn grandchild2(self) -> Self;
    fn grandchild3(self) -> Self;
    fn grandchild4(self) -> Self;

    fn has_parent(self) -> bool;
    fn has_grandparent(self) -> bool;
    fn is_min_level(self) -> bool;
}

impl HeapIndex for usize {
    #[inline]
    fn parent(self) -> Self { (self - 1) / 2 }
    #[inline]
    fn grandparent(self) -> Self { self.parent().parent() }

    #[inline]
    fn child1(self) -> Self { 2 * self + 1 }
    #[inline]
    fn child2(self) -> Self { 2 * self + 2 }

    #[inline]
    fn grandchild1(self) -> Self { self.child1().child1() }
    #[inline]
    fn grandchild2(self) -> Self { self.child1().child2() }
    #[inline]
    fn grandchild3(self) -> Self { self.child2().child1() }
    #[inline]
    fn grandchild4(self) -> Self { self.child2().child2() }

    #[inline]
    fn has_parent(self) -> bool {
        self > 0
    }

    #[inline]
    fn has_grandparent(self) -> bool {
        self > 2
    }

    #[inline]
    fn is_min_level(self) -> bool {
        (self + 1).leading_zeros() & 1 == 1
    }
}

//                       0
//           1                        2
//      3         4             5           6
//    7   8     9   10       11   12     13   14

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn t_parent() {
        assert_eq!(0, 1.parent());
        assert_eq!(0, 2.parent());
        assert_eq!(1, 3.parent());
        assert_eq!(1, 4.parent());
        assert_eq!(2, 5.parent());
        assert_eq!(2, 6.parent());
        assert_eq!(5, 11.parent());
    }

    #[test]
    fn t_child1() {
        assert_eq!(1, 0.child1());
        assert_eq!(3, 1.child1());
        assert_eq!(7, 3.child1());
        assert_eq!(13, 6.child1());
    }

    #[test]
    fn t_child2() {
        assert_eq!(2, 0.child2());
        assert_eq!(4, 1.child2());
        assert_eq!(8, 3.child2());
        assert_eq!(14, 6.child2());
    }

    #[test]
    fn t_grandchild1() {
        assert_eq!(3, 0.grandchild1());
        assert_eq!(7, 1.grandchild1());
        assert_eq!(11, 2.grandchild1());
    }

    #[test]
    fn t_grandchild2() {
        assert_eq!(4, 0.grandchild2());
        assert_eq!(8, 1.grandchild2());
        assert_eq!(12, 2.grandchild2());
    }

    #[test]
    fn t_grandchild3() {
        assert_eq!(5, 0.grandchild3());
        assert_eq!(9, 1.grandchild3());
        assert_eq!(13, 2.grandchild3());
    }

    #[test]
    fn t_grandchild4() {
        assert_eq!(6, 0.grandchild4());
        assert_eq!(10, 1.grandchild4());
        assert_eq!(14, 2.grandchild4());
    }

    #[test]
    fn t_has_parent() {
        assert!(! 0.has_parent());
        assert!(1.has_parent());
        assert!(2.has_parent());
        assert!(3.has_parent());
    }

    #[test]
    fn t_has_grandparent() {
        assert!(! 0.has_grandparent());
        assert!(! 1.has_grandparent());
        assert!(! 2.has_grandparent());
        assert!(3.has_grandparent());
        assert!(4.has_grandparent());
        assert!(5.has_grandparent());
        assert!(6.has_grandparent());
    }

    #[test]
    fn t_is_min_level() {
        assert!(0.is_min_level());
        assert!(!1.is_min_level());
        assert!(!2.is_min_level());
        assert!(3.is_min_level());
        assert!(4.is_min_level());
        assert!(5.is_min_level());
        assert!(6.is_min_level());
        assert!(!7.is_min_level());
        assert!(!8.is_min_level());
        assert!(!9.is_min_level());
        assert!(!10.is_min_level());
        assert!(!11.is_min_level());
        assert!(!12.is_min_level());
        assert!(!13.is_min_level());
        assert!(!14.is_min_level());
        assert!(15.is_min_level());
    }
}
