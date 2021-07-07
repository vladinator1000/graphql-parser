use crate::common::Text;

pub enum AstNode<'ast> {
    Name,
    Document,
    OperationDefinition,
    VariableDefinition,
    Variable,
    SelectionSet,
    Field,
    Argument,
    FragmentSpread,
    InlineFragment,
    FragmentDefinition,
    IntValue,
    FloatValue,
    StringValue,
    BooleanValue,
    NullValue,
    EnumValue,
    ListValue,
    ObjectValue,
    ObjectField,
    Directive,
    NamedType,
    ListType,
    NonNullType,
    SchemaDefinition,
    OperationTypeDefinition,
    ScalarTypeDefinition,
    ObjectTypeDefinition,
    FieldDefinition,
    InputValueDefinition,
    InterfaceTypeDefinition,
    UnionTypeDefinition,
    EnumTypeDefinition,
    EnumValueDefinition,
    InputObjectTypeDefinition,
    DirectiveDefinition,
    SchemaExtension,
    ScalarTypeExtension,
    ObjectTypeExtension,
    InterfaceTypeExtension,
    UnionTypeExtension,
    EnumTypeExtension,
    InputObjectTypeExtension,
}

// pub trait Visitor<'ast> {
//   enter(node: AstNode) {}
// }
