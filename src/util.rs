#[derive(Clone, Copy, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

pub fn rotate_forward(min: usize, max: usize, n: usize) -> usize {
    if n + 1 == max {
        min
    }
    else {
        n + 1
    }
}

pub fn rotate_backward(min: usize, max: usize, n: usize) -> usize {
    (if n == min { max } else { n }) - 1
}

#[cfg(test)]
mod tests {
    #[test]
    fn rotate_forward() {
        assert_eq!(super::rotate_forward(0, 3, 0), 1);
        assert_eq!(super::rotate_forward(0, 3, 1), 2);
        assert_eq!(super::rotate_forward(0, 3, 2), 0);
    }
    #[test]
    fn rotate_backward() {
        assert_eq!(super::rotate_backward(0, 3, 0), 2);
        assert_eq!(super::rotate_backward(0, 3, 1), 0);
        assert_eq!(super::rotate_backward(0, 3, 2), 1);
    }
}
