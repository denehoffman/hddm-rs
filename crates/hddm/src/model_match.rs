use crate::{
    error::{HddmError, HddmResult},
    header::{AttributeDef, ElementDef, HddmModel},
};

pub fn validate_models(input: &HddmModel, generated: &HddmModel) -> HddmResult<()> {
    if input.class_name != generated.class_name {
        return Err(HddmError::FormatError(format!(
            "HDDM class mismatch: file has {:?}, generated code expects {:?}",
            input.class_name, generated.class_name
        )));
    }

    validate_element(&input.root, &generated.root)
}

fn validate_element(input: &ElementDef, generated: &ElementDef) -> HddmResult<()> {
    if input.name != generated.name {
        return Err(HddmError::FormatError(format!(
            "HDDM element mismatch: file has {}, generated code expects {}",
            input.name, generated.name
        )));
    }

    validate_attributes(&input.attributes, &generated.attributes, &input.name)?;

    for generated_child in &generated.children {
        let matching_input = input
            .children
            .iter()
            .find(|child| child.name == generated_child.name);

        match matching_input {
            Some(input_child) => validate_element(input_child, generated_child)?,
            None => {
                if !is_optional_or_list(generated_child) {
                    return Err(HddmError::FormatError(format!(
                        "HDDM file is missing required child `{}` of `{}`",
                        generated_child.name, input.name
                    )));
                }
            }
        }
    }

    Ok(())
}

fn validate_attributes(
    input: &[AttributeDef],
    generated: &[AttributeDef],
    element: &str,
) -> HddmResult<()> {
    if input != generated {
        return Err(HddmError::FormatError(format!(
            "HDDM attribute mismatch in element `{}`: file has {:?}, generated code expects {:?}",
            element, input, generated
        )));
    }

    Ok(())
}

fn is_optional_or_list(elem: &ElementDef) -> bool {
    elem.min_occurs.as_deref() == Some("0") || elem.max_occurs.as_deref() == Some("unbounded")
}
