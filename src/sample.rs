use crate::data::{
    entity::{AccessModifier, Argument, Attribute, EntityType, Type},
    Entity, Project,
};

pub fn project() -> Project {
    let mut project = Project::new("Test".to_string());

    let mut basic = Entity::new("Basic".to_string(), EntityType::Class, (0, 4));

    basic.attributes.insert(
        "id".to_string(),
        Attribute::Field(
            AccessModifier::Private,
            Type::External("int".to_string(), vec![]),
        ),
    );

    basic.attributes.insert(
        "method".to_string(),
        Attribute::Method(
            AccessModifier::Public,
            None,
            Type::External("string".to_string(), vec![]),
            vec![Argument(
                "arg".to_string(),
                Type::External("int".to_string(), vec![]),
            )],
        ),
    );

    project.add_entity(basic);
    project.add_entity(Entity::new("Empty".to_string(), EntityType::AbstractClass, (0, 0)));

    project
}
