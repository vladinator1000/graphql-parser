//! TypeInfo is a utility class which, given a GraphQL schema, can keep track
//! of the current field and type definitions at any point in a GraphQL document
//! AST during a recursive descent by calling `enter(node)` and `leave(node)`.

use crate::{
    common::{CompositeType, InputType},
    query::Field,
    schema::{Directive, Document, EnumValue, Text, Type, Value},
};

#[derive(Debug)]
pub struct TypeInfo<'ast, T: Text<'ast>> {
    schema: Document<'ast, T>,
    type_stack: Vec<Option<Type<'ast, T>>>,
    parent_type_stack: Vec<Option<CompositeType<'ast, T>>>,
    input_type_stack: Vec<Option<InputType<'ast, T>>>,
    field_def_stack: Vec<Option<Field<'ast, T>>>,
    default_value_stack: Vec<Option<T>>,
    directive: Option<Directive<'ast, T>>,
    argument: Option<(T::Value, Value<'ast, T>)>,
    enum_value: Option<EnumValue<'ast, T>>,
}

impl<'ast, T: Text<'ast>> TypeInfo<'ast, T> {
    pub fn new(schema: Document<'ast, T>) -> TypeInfo<'ast, T> {
        TypeInfo {
            schema,
            type_stack: Vec::new(),
            parent_type_stack: Vec::new(),
            input_type_stack: Vec::new(),
            field_def_stack: Vec::new(),
            default_value_stack: Vec::new(),
            directive: None,
            argument: None,
            enum_value: None,
        }
    }

    pub fn get_type(&self) -> &Option<Type<'ast, T>> {
        let type_stack_length = self.type_stack.len();

        if type_stack_length > 0 {
            &self.type_stack[type_stack_length - 1]
        } else {
            &None
        }
    }

    pub fn get_parent_type(&self) -> &Option<CompositeType<'ast, T>> {
        let parent_type_stack_length = self.parent_type_stack.len();

        if parent_type_stack_length > 0 {
            &self.parent_type_stack[parent_type_stack_length - 1]
        } else {
            &None
        }
    }

    pub fn get_input_type(&self) -> &Option<InputType<'ast, T>> {
        let input_type_stack = self.input_type_stack.len();

        if input_type_stack > 0 {
            &self.input_type_stack[input_type_stack - 1]
        } else {
            &None
        }
    }
}

#[cfg(test)]
mod type_info_tests {
    use query::{Text, Type};

    use crate::{
        common::{CompositeType, InputType},
        parse_query, parse_schema,
        query::{self},
        schema::{self},
        visitor::type_info::TypeInfo,
    };

    const TEST_SCHEMA: &str = r#"
      interface Pet {
        name: String
      }

      type Dog implements Pet {
        name: String
      }

      type Cat implements Pet {
        name: String
      }

      type Human {
        name: String
        pets: [Pet]
      }

      type Alien {
        name(surname: Boolean): String
      }

      type QueryRoot {
        human(id: ID): Human
        alien: Alien
      }

      schema {
        query: QueryRoot
      }
    "#;

    #[test]
    pub fn visit_maintains_type_info() {
        let schema_ast: schema::Document<String> = parse_schema(TEST_SCHEMA).unwrap();
        let mut type_info = TypeInfo::new(schema_ast);

        let query_ast: query::Document<String> = parse_query(
            r#"{
            human(id: 4) {
              name,
              pets { 
                ... { name }
              },
              unknown
            } 
          }"#,
        )
        .unwrap();

        #[derive(Debug, PartialEq)]
        struct TestVisitor<'ast, T: Text<'ast>> {
            parent_types: Vec<&'ast Option<CompositeType<'ast, T>>>,
            current_types: Vec<&'ast Option<Type<'ast, T>>>,
            input_types: Vec<&'ast Option<InputType<'ast, T>>>,
        }

        impl<'ast, T: Text<'ast>> TestVisitor<'ast, T> {
            pub fn new() -> Self {
                Self {
                    parent_types: Vec::new(),
                    current_types: Vec::new(),
                    input_types: Vec::new(),
                }
            }
        }
    }
}
