use crate::data::{
    connection::{Multiplicity, Relation, RelationType},
    entity::{AccessModifier, Argument, Attribute, EntityType, Type},
    Connection, Entity, Project,
};

pub fn project() -> Project {
    let mut project = Project::new("Test".to_string());

    let pos1 = (0, 20);
    let mut basic = Entity::new("Basic".to_string(), EntityType::Class, pos1);

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
            Type::External("st".to_string(), vec![]),
            vec![Argument(
                "arg".to_string(),
                Type::External("int".to_string(), vec![]),
            )],
        ),
    );

    let pos2 = (0, 0);

    let conn = Connection::new(
        Relation {
            entity: project.add_entity(basic),
            relation: RelationType::Association,
            multiplicity: Multiplicity::One,
        },
        Relation {
            entity: project.add_entity(Entity::new(
                "Empty".to_string(),
                EntityType::AbstractClass,
                pos2,
            )),
            relation: RelationType::Association,
            multiplicity: Multiplicity::Many,
        },
        vec![(20, 20), (10, 13), (10, 7), (0, 0)],
        pos1.into(),
        pos2.into(),
    );

    project.connect(conn);

    project
}
