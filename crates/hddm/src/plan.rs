use crate::{
    error::{HddmError, HddmResult},
    header::{ElementDef, HddmModel},
};

#[derive(Debug, Clone)]
pub struct ModelPlan {
    pub root: ElementPlan,
}

#[derive(Debug, Clone)]
pub struct ElementPlan {
    pub element_name: String,
    pub children: Vec<ChildPlan>,
}

#[derive(Debug, Clone)]
pub enum ChildPlan {
    Decode {
        input_index: usize,
        generated_index: usize,
        plan: ElementPlan,
    },
    Skip {
        input_index: usize,
    },
}

pub fn build_model_plan(input: &HddmModel, generated: &HddmModel) -> HddmResult<ModelPlan> {
    if input.class_name != generated.class_name {
        return Err(HddmError::FormatError(format!(
            "HDDM class mismatch: file has {:?}, generated expects {:?}",
            input.class_name, generated.class_name
        )));
    }

    Ok(ModelPlan {
        root: build_element_plan(&input.root, &generated.root)?,
    })
}

fn build_element_plan(input: &ElementDef, generated: &ElementDef) -> HddmResult<ElementPlan> {
    if input.name != generated.name {
        return Err(HddmError::FormatError(format!(
            "HDDM element mismatch: file has {}, generated expects {}",
            input.name, generated.name
        )));
    }

    if input.attributes != generated.attributes {
        return Err(HddmError::FormatError(format!(
            "HDDM attribute mismatch in element `{}`",
            input.name
        )));
    }

    let mut children = Vec::new();

    for (input_index, input_child) in input.children.iter().enumerate() {
        if let Some((generated_index, generated_child)) = generated
            .children
            .iter()
            .enumerate()
            .find(|(_, child)| child.name == input_child.name)
        {
            children.push(ChildPlan::Decode {
                input_index,
                generated_index,
                plan: build_element_plan(input_child, generated_child)?,
            });
        } else {
            children.push(ChildPlan::Skip { input_index });
        }
    }

    for generated_child in &generated.children {
        let found = input
            .children
            .iter()
            .any(|child| child.name == generated_child.name);

        if !found && !is_optional_or_list(generated_child) {
            return Err(HddmError::FormatError(format!(
                "HDDM file is missing required child `{}` of `{}`",
                generated_child.name, input.name
            )));
        }
    }

    Ok(ElementPlan {
        element_name: input.name.clone(),
        children,
    })
}

fn is_optional_or_list(elem: &ElementDef) -> bool {
    elem.min_occurs.as_deref() == Some("0") || elem.max_occurs.as_deref() == Some("unbounded")
}
