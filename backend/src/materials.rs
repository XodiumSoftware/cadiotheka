//! IFC material association extraction.
//!
//! `ifc-lite-export` writes only render colors into the converted GLB, so an
//! object's material names have to be recovered from the source IFC bytes.
//! This module walks every `IfcRelAssociatesMaterial` entity and maps each
//! related element's express id to the names of its associated `IfcMaterial`
//! entities, covering single materials, material lists, layer sets (and their
//! usages), constituent sets, and profile sets.

use std::collections::HashMap;

use ifc_lite_core::{
    AttributeValue, DecodedEntity, EntityDecoder, EntityScanner, IfcType, build_entity_index,
};

/// Attribute index of `RelatedObjects` on `IfcRelAssociatesMaterial`.
const RELATED_OBJECTS_ATTR: usize = 4;
/// Attribute index of `RelatingMaterial` on `IfcRelAssociatesMaterial`.
const RELATING_MATERIAL_ATTR: usize = 5;
/// Recursion guard for the material select graph.
const MAX_SELECT_DEPTH: usize = 4;

/// Maps element express ids to the IFC material names assigned to them by the
/// model's `IfcRelAssociatesMaterial` relationships.
///
/// Names are deduplicated per element while preserving document order; elements
/// without a material association are absent from the map. Malformed or
/// unresolvable references are skipped so a broken model degrades to missing
/// data instead of an error.
pub fn extract_material_names(content: &[u8]) -> HashMap<u32, Vec<String>> {
    let index = build_entity_index(content);
    let mut decoder = EntityDecoder::with_index(content, index);
    let mut scanner = EntityScanner::new(content);
    let mut select_cache: HashMap<u32, Vec<String>> = HashMap::new();
    let mut names_by_element: HashMap<u32, Vec<String>> = HashMap::new();

    while let Some((id, type_name, _, _)) = scanner.next_entity() {
        if !type_name.eq_ignore_ascii_case("IFCRELASSOCIATESMATERIAL") {
            continue;
        }
        let Ok(rel) = decoder.decode_by_id(id) else {
            continue;
        };
        let Some(select_id) = rel.get_ref(RELATING_MATERIAL_ATTR) else {
            continue;
        };
        let names = select_cache.entry(select_id).or_insert_with(|| {
            let mut names = Vec::new();
            collect_select_names(select_id, &mut decoder, 0, &mut names);
            names
        });
        if names.is_empty() {
            continue;
        }
        let Some(objects) = rel.get_list(RELATED_OBJECTS_ATTR) else {
            continue;
        };
        let element_ids: Vec<u32> = objects
            .iter()
            .filter_map(AttributeValue::as_entity_ref)
            .collect();
        for element_id in element_ids {
            let entry = names_by_element.entry(element_id).or_default();
            for name in names.iter() {
                if !entry.contains(name) {
                    entry.push(name.clone());
                }
            }
        }
    }

    names_by_element
}

/// Resolves the material names reachable from one `RelatingMaterial` select
/// entity, recursing into material lists, layer sets, constituent sets, and
/// profile sets to reach the underlying `IfcMaterial` names.
fn collect_select_names(
    select_id: u32,
    decoder: &mut EntityDecoder<'_>,
    depth: usize,
    names: &mut Vec<String>,
) {
    if depth > MAX_SELECT_DEPTH {
        return;
    }
    let Ok(entity) = decoder.decode_by_id(select_id) else {
        return;
    };
    match entity.ifc_type {
        IfcType::IfcMaterial => {
            if let Some(name) = entity.get_string(0) {
                push_unique(names, name);
            }
        }
        IfcType::IfcMaterialList | IfcType::IfcMaterialLayerSet => {
            collect_reference_list(decoder, &entity, 0, depth, names);
        }
        IfcType::IfcMaterialLayer
        | IfcType::IfcMaterialLayerSetUsage
        | IfcType::IfcMaterialProfileSetUsage
        | IfcType::IfcMaterialProfileSetUsageTapering => {
            collect_reference(decoder, &entity, 0, depth, names);
        }
        IfcType::IfcMaterialConstituent | IfcType::IfcMaterialProfile => {
            collect_reference(decoder, &entity, 2, depth, names);
        }
        IfcType::IfcMaterialConstituentSet | IfcType::IfcMaterialProfileSet => {
            collect_reference_list(decoder, &entity, 2, depth, names);
        }
        _ => {}
    }
}

/// Recurses into every material entity referenced by a list attribute.
fn collect_reference_list(
    decoder: &mut EntityDecoder<'_>,
    entity: &DecodedEntity,
    attribute: usize,
    depth: usize,
    names: &mut Vec<String>,
) {
    let Some(items) = entity.get_list(attribute) else {
        return;
    };
    let references: Vec<u32> = items
        .iter()
        .filter_map(AttributeValue::as_entity_ref)
        .collect();
    for reference in references {
        collect_select_names(reference, decoder, depth + 1, names);
    }
}

/// Recurses into the material entity referenced by a single attribute.
fn collect_reference(
    decoder: &mut EntityDecoder<'_>,
    entity: &DecodedEntity,
    attribute: usize,
    depth: usize,
    names: &mut Vec<String>,
) {
    let Some(reference) = entity.get_ref(attribute) else {
        return;
    };
    collect_select_names(reference, decoder, depth + 1, names);
}

/// Appends `name` to `names` unless already present.
fn push_unique(names: &mut Vec<String>, name: &str) {
    if !names.iter().any(|existing| existing == name) {
        names.push(name.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::extract_material_names;

    const IFC: &str = "#1=IFCMATERIAL('Concrete',$,$);\n\
        #2=IFCMATERIAL('Steel',$,$);\n\
        #3=IFCMATERIALLIST((#1,#2));\n\
        #4=IFCMATERIALLAYER(#1,0.2,$);\n\
        #5=IFCMATERIALLAYERSET((#4),'Wall assembly');\n\
        #6=IFCMATERIALLAYERSETUSAGE(#5,.AXIS2.,.POSITIVE.,-0.1);\n\
        #7=IFCMATERIALCONSTITUENT('Rebar',$,#2,$);\n\
        #8=IFCMATERIALCONSTITUENTSET('Mix',$, (#7));\n\
        #9=IFCRELASSOCIATESMATERIAL('guid-a',$,$,$,(#100),#1);\n\
        #10=IFCRELASSOCIATESMATERIAL('guid-b',$,$,$,(#101,#102),#3);\n\
        #11=IFCRELASSOCIATESMATERIAL('guid-c',$,$,$,(#103),#6);\n\
        #12=IFCRELASSOCIATESMATERIAL('guid-d',$,$,$,(#104),#8);\n\
        #13=IFCRELASSOCIATESMATERIAL('guid-e',$,$,$,(#105),$);\n";

    #[test]
    fn single_material_association_resolves_name() {
        let map = extract_material_names(IFC.as_bytes());
        assert_eq!(map.get(&100), Some(&vec!["Concrete".to_string()]));
    }

    #[test]
    fn material_list_assigns_all_names_to_each_element() {
        let map = extract_material_names(IFC.as_bytes());
        let expected = vec!["Concrete".to_string(), "Steel".to_string()];
        assert_eq!(map.get(&101), Some(&expected));
        assert_eq!(map.get(&102), Some(&expected));
    }

    #[test]
    fn layer_set_usage_resolves_layer_materials() {
        let map = extract_material_names(IFC.as_bytes());
        assert_eq!(map.get(&103), Some(&vec!["Concrete".to_string()]));
    }

    #[test]
    fn constituent_set_resolves_constituent_materials() {
        let map = extract_material_names(IFC.as_bytes());
        assert_eq!(map.get(&104), Some(&vec!["Steel".to_string()]));
    }

    #[test]
    fn null_relating_material_yields_no_entry() {
        let map = extract_material_names(IFC.as_bytes());
        assert!(!map.contains_key(&105));
    }

    #[test]
    fn empty_content_yields_empty_map() {
        assert!(extract_material_names(b"").is_empty());
    }
}
