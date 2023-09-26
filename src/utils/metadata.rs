use crate::utils::sampler3d;

use crate::widgets::key_value_list_widget::KeyValueList;

pub fn make_metadata_key_value_list(
    image_sampler: &sampler3d::Sampler3D,
) -> KeyValueList {
    let ndim = image_sampler.header.dim[0] as usize;
    vec![
        (
            "Data type".to_owned(),
            format!("{:?}", image_sampler.header.data_type().unwrap()),
        ),
        (
            "Ndim".to_owned(),
            format!("{}", image_sampler.header.dim[0]),
        ),
        (
            "Shape".to_owned(),
            format!("{:?}", &image_sampler.header.dim[1..ndim + 1]),
        ),
        (
            "Units".to_owned(),
            format!(
                "{:?} (space); {:?} (time)",
                &image_sampler
                    .header
                    .xyzt_to_space()
                    .unwrap_or(nifti::Unit::Unknown),
                &image_sampler
                    .header
                    .xyzt_to_time()
                    .unwrap_or(nifti::Unit::Unknown)
            ),
        ),
        (
            "Data scaling".to_owned(),
            format!(
                "{} + {} * x",
                image_sampler.header.scl_inter, image_sampler.header.scl_slope
            ),
        ),
        (
            "Display range".to_owned(),
            format!(
                "[{}, {}]",
                image_sampler.header.cal_min, image_sampler.header.cal_max
            ),
        ),
        (
            "Description".to_owned(),
            format!(
                "'{}'",
                String::from_utf8(image_sampler.header.descrip.clone())
                    .unwrap_or("<error>".to_owned())
            ),
        ),
        (
            "Intent".to_owned(),
            format!(
                "'{}'",
                String::from_utf8(image_sampler.header.intent_name.to_vec())
                    .unwrap_or("<error>".to_owned())
            ),
        ),
        (
            "Slice order".to_owned(),
            format!(
                "{:?}",
                &image_sampler
                    .header
                    .slice_order()
                    .unwrap_or(nifti::SliceOrder::Unknown)
            ),
        ),
        (
            "Slice duration".to_owned(),
            format!("{}", image_sampler.header.slice_duration),
        ),
        (
            "Affine".to_owned(),
            format!("{:?}", &image_sampler.header.srow_x),
        ),
        ("".to_owned(), format!("{:?}", &image_sampler.header.srow_y)),
        ("".to_owned(), format!("{:?}", &image_sampler.header.srow_z)),
    ]
}