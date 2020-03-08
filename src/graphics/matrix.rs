#![allow(dead_code)]

use super::parametrics::Parametric;
use super::utils;
use std::f64::consts;
use std::fmt;


#[derive(Clone, Debug)]
/// Row major rectangular matrix
/// Each row represents a new point
pub struct Matrix {
    nrows: usize,
    ncols: usize,
    data: Vec<f64>,
}

// constructor, get, set
impl Matrix {
    /// Row major index
    fn index(&self, row: usize, col: usize) -> usize {
        row * self.ncols + col
        // col * self.nrows + row
    }

    pub fn new_clone_vec(nrows: usize, ncols: usize, data: &Vec<f64>) -> Matrix {
        assert_eq!(
            nrows * ncols,
            data.len(),
            "nrows * ncols must == data.len()"
        );

        Matrix {
            nrows,
            ncols,
            data: data.clone(),
        }
    }

    pub fn new(nrows: usize, ncols: usize, data: Vec<f64>) -> Matrix {
        assert_eq!(
            nrows * ncols,
            data.len(),
            "nrows * ncols must == data.len()"
        );
        Matrix { nrows, ncols, data }
    }

    pub fn get(&self, row: usize, col: usize) -> Option<f64> {
        if row > self.nrows || col > self.ncols {
            None
        } else {
            Some(self.data[self.index(row, col)])
        }
    }

    pub fn set(&mut self, row: usize, col: usize, data: f64) {
        assert!(row < self.nrows && col < self.ncols, "Index out of bound");
        let i = self.index(row, col);
        self.data[i] = data;
    }
}

// add edge (row)
impl Matrix {
    pub fn append_row(&mut self, row: &mut Vec<f64>) {
        assert_eq!(
            self.ncols,
            row.len(),
            "Length of edge and matrix column size don't match"
        );
        self.data.append(row);
        self.nrows += 1;
    }

    /// Append an edge in the format [x0, y0, z0, x1, y1, z1]
    pub fn append_edge(&mut self, edge: &Vec<f64>) {
        assert_eq!(6, edge.len(), "Len of edge vec should be 6");
        self.data.extend_from_slice(&edge[0..3]);
        self.data.push(1.0);
        self.data.extend_from_slice(&edge[3..6]);
        self.data.push(1.0);
        self.nrows += 2;
    }
}

// row and col iter
impl Matrix {
    /// Iterate over a certain row
    pub fn row_iter<'a>(&'a self, r: usize) -> impl Iterator<Item = &f64> {
        let start = r * self.ncols;
        self.data[start..start + self.ncols].iter()
    }

    /// Iterate over a certain column
    pub fn col_iter<'a>(&'a self, c: usize) -> impl Iterator<Item = &f64> {
        self.data.iter().skip(c).step_by(self.ncols)
    }

    /// Interate over the matrix by row, one row at a time
    ///
    /// Returns an iterator for the row
    pub fn iter_by_row(&self) -> std::slice::Chunks<'_, f64> {
        self.data.as_slice().chunks(self.ncols)
    }
}

// mul
impl Matrix {
    /// Returns (x, y) of a matrix based on ncols and i
    fn index_to_rc(i: usize, ncols: usize) -> (usize, usize) {
        (i / ncols, i % ncols)
    }

    /// Multiplies self matrix by other matrix
    pub fn mul(&self, other: &Self) -> Self {
        // self * other -> new
        assert_eq!(self.ncols, other.nrows, "ncols of m1 must == nrows of m2");
        let (frows, fcols) = (self.nrows, other.ncols);
        let mut fdata = vec![0.0; frows * fcols];
        for (i, d) in fdata.iter_mut().enumerate() {
            let (r, c) = Self::index_to_rc(i, fcols);
            *d = self
                .row_iter(r)
                .zip(other.col_iter(c))
                .fold(0.0, |sum, (a, b)| sum + a * b);
        }
        Matrix::new(frows, fcols, fdata)
    }

    pub fn transposed_mul(&self, other: &Self) -> Self {
        assert_eq!(self.nrows, other.ncols, "nrows of m1 must == ncols of m2");
        let (frows, fcols) = (other.nrows, self.nrows);
        let mut fdata = vec![0.0; frows * fcols];
        for (i, d) in fdata.iter_mut().enumerate() {
            let (r, c) = Self::index_to_rc(i, fcols);
            *d = self
                .col_iter(c)
                .zip(other.row_iter(r))
                .fold(0.0, |sum, (a, b)| sum + a * b);
        }
        Matrix::new(frows, fcols, fdata)
    }

    pub fn mul_mut_b(a: &Matrix, b: &mut Matrix) {
        *b = a.mul(b);
        // println!("result: {}", b);
    }
}

// identity
impl Matrix {
    /// Make a new identity matrix with size `size`
    pub fn ident(size: usize) -> Self {
        let mut m = Matrix::new(size, size, vec![0.0; size * size]);
        for i in 0..size {
            m.set(i, i, 1.0);
        }
        m
    }

    /// Transforms self into an identity matrix
    pub fn to_ident(&mut self) {
        let ncols = self.ncols;
        for (i, d) in self.data.iter_mut().enumerate() {
            *d = if {
                let (r, c) = Matrix::index_to_rc(i, ncols);
                r == c
            } {
                1.0
            } else {
                0.0
            }
        }
    }
}

// generate transformation matrices
impl Matrix {
    /// Generate a translate matrix with (dx, dy, dz)
    pub fn mv(dx: f64, dy: f64, dz: f64) -> Matrix {
        let mut m = Matrix::ident(4);

        m.set(3, 0, dx);
        m.set(3, 1, dy);
        m.set(3, 2, dz);
        m
    }

    /// Generate a scale matrix with (sx, sy, sz)
    pub fn scale(sx: f64, sy: f64, sz: f64) -> Matrix {
        let mut m = Matrix::ident(4);
        m.set(0, 0, sx);
        m.set(1, 1, sy);
        m.set(2, 2, sz);
        m
    }

    
    #[rustfmt::skip]
    pub fn rotatex(angle_deg: f64) -> Matrix {
        // let mut m = Matrix::ident(4);
        // m.set(1, 1, angle_deg.to_radians().cos());
        // m.set(2, 2, angle_deg.to_radians().cos());
        // m.set(1, 2, -angle_deg.to_radians().sin());
        // m.set(2, 1, angle_deg.to_radians().sin());
        // m
        let a = angle_deg.to_radians();
        Matrix::new(
            4,
            4,
            vec![ 
                1.0, 0.0, 0.0, 0.0, 
                0.0, a.cos(), -a.sin(), 0.0, 0.0, 
                a.sin(), a.cos(), 0.0, 
                0.0, 0.0, 0.0, 1.0,
            ],
        )
    }

    #[rustfmt::skip]
    pub fn rotatey(angle_deg: f64) -> Matrix {
        // let mut m = Matrix::ident(4);
        // m.set(0, 0, angle_deg.to_radians().cos());
        // m.set(0, 2, angle_deg.to_radians().sin());
        // m.set(2, 0, -angle_deg.to_radians().sin());
        // m.set(2, 2, angle_deg.to_radians().cos());
        // m
        let a = angle_deg.to_radians();
        Matrix::new(
            4,
            4,
            vec![ 
                a.cos(), 0.0, a.sin(), 0.0, 
                0.0, 1.0, 0.0, 0.0, 
                -a.sin(), 0.0, a.cos(), 0.0, 
                0.0, 0.0, 0.0, 1.0,
            ],
        )
    }

    pub fn rotatez(angle_deg: f64) -> Matrix {
        let mut m = Matrix::ident(4);
        m.set(0, 0, angle_deg.to_radians().cos());
        m.set(1, 1, angle_deg.to_radians().cos());
        m.set(1, 0, angle_deg.to_radians().sin());
        m.set(0, 1, -angle_deg.to_radians().sin());
        m
    }
}

// print Matrix
impl fmt::Display for Matrix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.nrows == 0 || self.ncols == 0 {
            write!(f, "Empty matrix ({} by {})", self.nrows, self.ncols)?;
        } else {
            writeln!(f, "Matrix ({} by {}) {{", self.nrows, self.ncols)?;

            for col_offset in 0..self.ncols {
                write!(f, "  ")?; // indentation
                for d in self.data.iter().skip(col_offset).step_by(self.ncols) {
                    write!(f, "{arg:.prec$} ", arg = d, prec = 2)?;
                }
                writeln!(f)?; // line change
            }
            write!(f, "}}")?;
        }
        Ok(())
    }
}

// draw parametric
impl Matrix {
    /// Add a parametric curve
    /// # Arguments
    /// `x` - Function that takes in `t` from 0 to 1 and produces x
    /// `y` - Function that takes in `t` from 0 to 1 and produces y
    /// `z` - The z value that the curve will be on
    /// `step` - Controls the precision of the curves
    pub fn add_parametric<F1, F2>(&mut self, xf: F1, yf: F2, z: f64, step: f64)
    where
        F1: Fn(f64) -> f64,
        F2: Fn(f64) -> f64,
    {
        let p = Parametric::new(xf, yf);
        for points in p.points_iter(step).collect::<Vec<(f64, f64)>>().windows(2) {
            let (x0, y0) = points[0];
            let (x1, y1) = points[1];
            self.append_edge(&vec![x0, y0, z, x1, y1, z]);
        }
    }

    /// Add a circle with center c `(x, y, z)` and radius `r`
    pub fn add_circle(&mut self, c: (f64, f64, f64), r: f64) {
        let (x, y, z) = c;
        self.add_parametric(
            |t: f64| r * (t * 2.0 * consts::PI).cos() + x,
            |t: f64| r * (t * 2.0 * consts::PI).sin() + y,
            z,
            0.001,
        );
    }


    /// Add a cubic Bezier curve
    /// # Arguments
    /// `p[0-3]` - control points
    pub fn add_bezier3(&mut self, p0: (f64, f64), p1: (f64, f64), p2: (f64, f64), p3: (f64, f64)) {

        let (ax, bx, cx, dx) = utils::compute_bezier3_coef(p0.0, p1.0, p2.0, p3.0);
        let (ay, by, cy, dy) = utils::compute_bezier3_coef(p0.1, p1.1, p2.1, p3.1);
        self.add_parametric(
            |t: f64| ax * t*t*t + bx * t * t + cx * t + dx, 
            |t: f64| ay * t*t*t + by * t * t + cy * t + dy, 
            0.0, 0.001);
    }

    pub fn add_hermite3(&mut self, p0: (f64, f64), p1: (f64, f64), r0: (f64, f64), r1: (f64, f64)) {
        
        let (ax, bx, cx, dx) = utils::compute_hermite3_coef(p0.0, p1.0, r0.0, r1.0);
        let (ay, by, cy, dy) = utils::compute_hermite3_coef(p0.1, p1.1, r0.1, r1.1);
        self.add_parametric(
            |t: f64| ax * t*t*t + bx * t * t + cx * t + dx, 
            |t: f64| ay * t*t*t + by * t * t + cy * t + dy, 
            0.0, 0.0001);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn matrix_equal(m1: &Matrix, m2: &Matrix) -> bool {
        m1.nrows == m2.nrows
            && m1.ncols == m2.ncols
            && m1.data.iter().zip(m2.data.iter()).all(|(a, b)| a == b)
    }

    #[test]
    #[ignore]
    fn print_matrix() {
        let m = Matrix::new(
            7,
            5,
            vec![
                1.0, 2.0, 3.0, 4.0, 5.0, 1.0, 2.0, 3.0, 4.0, 5.0, 1.0, 2.0, 3.0, 4.0, 5.0, 1.0,
                2.0, 3.0, 4.0, 5.0, 1.0, 2.0, 3.0, 4.0, 5.0, 1.0, 2.0, 3.0, 4.0, 5.0, 1.0, 2.0,
                3.0, 4.0, 5.0,
            ],
        );
        println!("M: {}", m);
        println!("M: {:?}", m);
    }

    #[test]
    fn add_edge() {
        let mut m = Matrix::new(0, 4, vec![]);
        println!("m: {}", m);
        println!("Adding (1, 2, 4) and (5, 6, 7) to empty matrix",);
        m.append_edge(&mut vec![1.0, 2.0, 4.0]);
        m.append_edge(&mut vec![5.0, 6.0, 7.0]);
        println!("m: {}", m);
        assert!(
            matrix_equal(
                &m,
                &Matrix::new(2, 4, vec![1.0, 2.0, 4.0, 1.0, 5.0, 6.0, 7.0, 1.0,])
            ),
            "Matrix not equal"
        );
    }

    #[test]
    fn multiply_with_method() {
        let m1 = Matrix::new(2, 3, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
        let m2 = Matrix::new(3, 2, vec![7.0, 8.0, 9.0, 10.0, 11.0, 12.0]);
        let mp = m1.mul(&m2);
        println!("{} mul by {} = {}", m1, m2, m1.mul(&m2));
        assert!(matrix_equal(
            &mp,
            &Matrix::new(2, 2, vec![58.0, 64.0, 139.0, 154.0,])
        ));
    }

    #[test]
    fn multiple_and_mutate_b() {
        let a = Matrix::new(1, 3, vec![3.0, 4.0, 2.0]);
        let mut b = Matrix::new(
            3,
            4,
            vec![13.0, 9.0, 7.0, 15.0, 8.0, 7.0, 4.0, 6.0, 6.0, 4.0, 0.0, 3.0],
        );
        println!("a: {}", a);
        println!("b: {}", b);
        println!("multiplying...",);
        Matrix::mul_mut_b(&a, &mut b);
        println!("b: {}", b);
        assert!(matrix_equal(
            &b,
            &Matrix::new(1, 4, vec![83.0, 63.0, 37.0, 75.0])
        ));
    }

    #[test]
    fn test_new_ident() {
        let ident = Matrix::ident(3);
        assert!(
            matrix_equal(
                &ident,
                &Matrix::new(3, 3, vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0,])
            ),
            "3 x 3 matrix"
        );

        assert!(
            matrix_equal(&Matrix::ident(1), &Matrix::new(1, 1, vec![1.0])),
            "1 x 1 matrix edge case"
        );
    }

    #[test]
    fn test_inplace_ident() {
        let mut m = Matrix::new(5, 5, vec![120.0; 25]);
        println!("m init: {}", m);
        println!("Mutating m...",);
        m.to_ident();
        println!("m is now {}", m);
        assert!(matrix_equal(&m, &Matrix::ident(5)), "5 x 5 matrix");
        let mut m = Matrix::new(1, 1, vec![50.0]);
        m.to_ident();
        assert!(
            matrix_equal(&m, &Matrix::ident(1)),
            "1 x 1 matrix edge case"
        );
    }
}
