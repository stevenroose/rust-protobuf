use protobuf::Message;
use protobuf::reflect::ReflectValueBox;
use protobuf::reflect::FieldDescriptor;

use super::test_reflect_pb::*;
use protobuf::reflect::RuntimeFieldType;

use protobuf_test_common::value_for_runtime_type;


#[test]
fn test_get_sub_message_via_reflection() {
    let mut m = M::new();
    m.mut_sub_m().set_n(42);
    assert!(m.has_sub_m());

    let descriptor = m.descriptor().field_by_name("sub_m").unwrap();
    assert_eq!("sub_m", descriptor.name());

    let sub_m = descriptor.get_message(&m);
    assert_eq!("test_reflect.SubM", sub_m.descriptor().full_name());
    assert_eq!(42, sub_m.descriptor().field_by_name("n").unwrap().get_i32(sub_m));
}

#[test]
fn test_singular_basic() {
    let mut message = TestTypesSingular::new();
    let descriptor = message.descriptor();

    let bool_field = descriptor.field_by_name("bool_field").unwrap();
    assert!(!bool_field.has_field(&message));

    bool_field.set_singular_field(&mut message, ReflectValueBox::Bool(true));
    assert!(bool_field.has_field(&message));
    assert_eq!(true, bool_field.get_bool(&message));
}

fn test_singular_field(message: &mut Message, field: &FieldDescriptor) {
    assert!(!field.has_field(message));

    // should not crash
    field.get_singular_field_or_default(message);

    let value = value_for_runtime_type(field.singular_runtime_type());
    field.set_singular_field(message, value);
}

#[test]
fn test_singular() {
    let mut message = TestTypesSingular::new();
    let descriptor = message.descriptor();

    for field in descriptor.fields() {
        test_singular_field(&mut message, field);
    }
}

#[test]
fn test_repeated_debug() {
    let mut message = TestTypesRepeated::new();
    message.set_int32_field(vec![10, 20, 30]);
    let field = message.descriptor().field_by_name("int32_field").unwrap().get_repeated(&message);
    assert_eq!("[10, 20, 30]", format!("{:?}", field));
}

fn test_repeated_field(message: &mut Message, field: &FieldDescriptor) {
    assert_eq!(0, field.len_field(message));
    assert!(!field.has_field(message));

    let mut expected = Vec::new();

    // test mut interface
    {
        let mut repeated = field.mut_repeated(message);

        for i in 0..3 {
            let value = value_for_runtime_type(repeated.element_type());
            expected.push(value.clone());
            repeated.push(value.clone());
            let fetched = repeated.get(i);
            assert_eq!(value, fetched);
        }

        assert_eq!(expected, repeated);
        assert_eq!(repeated, expected);
    }

    // test read interface
    {
        let repeated = field.get_repeated(message);
        assert_eq!(3, repeated.len());

        assert_eq!(expected, repeated);
        assert_eq!(repeated, expected);
    }
}

#[test]
fn test_repeated() {
    let mut message = TestTypesRepeated::new();
    let descriptor = message.descriptor();

    for field in descriptor.fields() {
        test_repeated_field(&mut message, field);
    }
}


fn test_map_field(message: &mut Message, field: &FieldDescriptor) {
    assert!(field.get_map(message).is_empty());
    assert_eq!(0, field.get_map(message).len());
    assert!(field.mut_map(message).is_empty());
    assert_eq!(0, field.mut_map(message).len());

    let (k, v) = match field.runtime_field_type() {
        RuntimeFieldType::Map(k, v) => (k, v),
        _ => panic!("not a map"),
    };

    {
        let map = field.get_map(message);
        assert!(map.is_empty());
        assert_eq!(0, map.len());

        assert_eq!(None, map.get(value_for_runtime_type(k).as_value_ref()));
    }

    {
        let mut map = field.mut_map(message);
        assert!(map.is_empty());
        assert_eq!(0, map.len());

        assert_eq!(None, map.get(value_for_runtime_type(k).as_value_ref()));

        let key = value_for_runtime_type(k);
        let value = value_for_runtime_type(v);

        map.insert(key.clone(), value.clone());

        assert_eq!(Some(value.as_value_ref()), map.get(key.as_value_ref()));

        assert_eq!(1, map.len());
    }
}

#[test]
fn test_map() {
    let mut message = TestTypesMap::new();
    let descriptor = message.descriptor();

    for field in descriptor.fields() {
        test_map_field(&mut message, field);
    }
}
