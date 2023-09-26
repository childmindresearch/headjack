use argminmax::ArgMinMax;

pub trait MinMax2<T> {
    fn minmax2(&self) -> (T, T);
}

impl<D> MinMax2<f32> for ndarray::Array<f32, D>
where
    D: ndarray::Dimension,
{
    fn minmax2(&self) -> (f32, f32) {
        let con = self.as_standard_layout();
        let contiguous = con.as_slice().unwrap();
        let (pmin, pmax) = contiguous.argminmax();
        return (contiguous[pmin], contiguous[pmax]);
    }
}
impl<D> MinMax2<f64> for ndarray::Array<f64, D>
where
    D: ndarray::Dimension,
{
    fn minmax2(&self) -> (f64, f64) {
        let con = self.as_standard_layout();
        let contiguous = con.as_slice().unwrap();
        let (pmin, pmax) = contiguous.argminmax();
        return (contiguous[pmin], contiguous[pmax]);
    }
}
impl<D> MinMax2<u16> for ndarray::Array<u16, D>
where
    D: ndarray::Dimension,
{
    fn minmax2(&self) -> (u16, u16) {
        let con = self.as_standard_layout();
        let contiguous = con.as_slice().unwrap();
        let (pmin, pmax) = contiguous.argminmax();
        return (contiguous[pmin], contiguous[pmax]);
    }
}
impl<D> MinMax2<u32> for ndarray::Array<u32, D>
where
    D: ndarray::Dimension,
{
    fn minmax2(&self) -> (u32, u32) {
        let con = self.as_standard_layout();
        let contiguous = con.as_slice().unwrap();
        let (pmin, pmax) = contiguous.argminmax();
        return (contiguous[pmin], contiguous[pmax]);
    }
}
impl<D> MinMax2<u64> for ndarray::Array<u64, D>
where
    D: ndarray::Dimension,
{
    fn minmax2(&self) -> (u64, u64) {
        let con = self.as_standard_layout();
        let contiguous = con.as_slice().unwrap();
        let (pmin, pmax) = contiguous.argminmax();
        return (contiguous[pmin], contiguous[pmax]);
    }
}
