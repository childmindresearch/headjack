use crate::utils::brain_volume::BrainMetaData;

use crate::widgets::key_value_list_widget::KeyValueList;

pub fn make_metadata_key_value_list(
    header: &BrainMetaData,
) -> KeyValueList {
    let ndim = header.dim[0] as usize;
    vec![
        (
            "Data type".to_owned(),
            format!("{:?}", header.data_type().unwrap()),
        ),
        (
            "Ndim".to_owned(),
            format!("{}", header.dim[0]),
        ),
        (
            "Shape".to_owned(),
            format!("{:?}", &header.dim[1..ndim + 1]),
        ),
        (
            "Units".to_owned(),
            format!(
                "{:?} (space); {:?} (time)",
                &header
                    .xyzt_to_space()
                    .unwrap_or(nifti::Unit::Unknown),
                &header
                    .xyzt_to_time()
                    .unwrap_or(nifti::Unit::Unknown)
            ),
        ),
        (
            "Data scaling".to_owned(),
            format!(
                "{} + {} * x",
                header.scl_inter, header.scl_slope
            ),
        ),
        (
            "Display range".to_owned(),
            format!(
                "[{}, {}]",
                header.cal_min, header.cal_max
            ),
        ),
        (
            "Description".to_owned(),
            format!(
                "'{}'",
                String::from_utf8(header.descrip.clone())
                    .unwrap_or("<error>".to_owned())
            ),
        ),
        (
            "Intent".to_owned(),
            format!(
                "'{}'",
                String::from_utf8(header.intent_name.to_vec())
                    .unwrap_or("<error>".to_owned())
            ),
        ),
        (
            "Slice order".to_owned(),
            format!(
                "{:?}",
                &header
                    .slice_order()
                    .unwrap_or(nifti::SliceOrder::Unknown)
            ),
        ),
        (
            "Slice duration".to_owned(),
            format!("{}", header.slice_duration),
        ),
        (
            "Affine".to_owned(),
            format!("{:?}", &header.srow_x),
        ),
        ("".to_owned(), format!("{:?}", &header.srow_y)),
        ("".to_owned(), format!("{:?}", &header.srow_z)),
        (
            "Grid spacings".to_owned(),
            format!("{:?}", &header.pixdim[1..ndim + 1]),
        ),
        (
            "Grid offsets".to_owned(),
            format!("{},{},{}", header.quatern_x, header.quatern_y, header.quatern_z),
        ),
    ]
}