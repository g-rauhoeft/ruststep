initSidebarItems({"enum":[["AnchorItem",""],["EntityInstance",""],["Parameter",""],["UntypedParameter",""]],"fn":[["anchor","anchor = [anchor_name] `=` [anchor_item] { [anchor_tag] } `;` ."],["anchor_item","anchor_item = `$` | [integer] | [real] | [string] | [enumeration] | binary | [rhs_occurrence_name] | [resource] | [anchor_item_list] ."],["anchor_item_list","anchor_item_list = `(` [ [anchor_item] { `,` [anchor_item] } ] `)` ."],["anchor_list","anchor_list = { [anchor()] } ."],["anchor_section","anchor_section = `ANCHOR;` [anchor_list] `ENDSEC;` ."],["anchor_tag","anchor_tag = `{` [tag_name] `:` [anchor_item] `}` ."],["complex_entity_instance","complex_entity_instance = [entity_instance_name] `=` [subsuper_record] `;` ."],["data_section","data_section = `DATA` [ `(` [parameter_list] `)` ] `;` [entity_instance_list] `ENDSEC;` ."],["entity_instance","entity_instance = [simple_entity_instance] | [complex_entity_instance] ."],["entity_instance_list","entity_instance_list = { [entity_instance] } ."],["exchange_file","exchange_file = `ISO-10303-21;` [header_section] [ [anchor_section] ] [ [reference_section] ] { [data_section] } `END-ISO-10303-21;` { signature_section } ."],["header_entity","header_entity = [simple_record] `;` ."],["header_entity_list","header_entity_list = [header_entity] { [header_entity] } ."],["header_section","header_section = `HEADER;` [header_entity] [header_entity] [header_entity] [ [header_entity_list] ] `ENDSEC;` ."],["list","list = `(` [ [parameter] { `,` [parameter] } ] `)` ."],["omitted_parameter","omitted_parameter = `*` ."],["parameter","parameter = [typed_parameter] | [untyped_parameter] | [omitted_parameter] ."],["parameter_list","parameter_list = [parameter] { `,` [parameter] } ."],["reference","reference = [lhs_occurrence_name] `=` [resource] `;` ."],["reference_list","reference_list = { [reference] } ."],["reference_section","reference_section = `REFERENCE;` [reference_list] `ENDSEC;` ."],["signature_section","signature_section  = `SIGNATURE` signature_content `ENDSEC;`."],["simple_entity_instance","simple_entity_instance = [entity_instance_name] `=` [simple_record] `;` ."],["simple_record","simple_record = [keyword] `(` [ [parameter_list] ] `)` ."],["simple_record_list","simple_record_list = [simple_record] { [simple_record] } ."],["subsuper_record","subsuper_record = `(` [simple_record_list] `)` ."],["typed_parameter","typed_parameter = [keyword] `(` [parameter] `)` ."],["untyped_parameter","untyped_parameter = `$` | [integer] | [real] | [string] | [rhs_occurrence_name] | [enumeration] | binary | [list] ."]],"struct":[["Anchor",""],["DataSection",""],["Exchange",""],["Record",""],["ReferenceEntry",""]]});