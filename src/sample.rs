use crate::data::{
    connection::{Multiplicity, Relation, RelationType},
    entity::{AccessModifier, Argument, Attribute, EntityType, Type},
    Connection, Entity, Project,
};
use crate::geometry::{Rect, Size};

pub fn project() -> Project {
    let mut project = Project::new("Test".to_string());

    let pos1 = (-5, 20);
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

    let pos2 = (13, 15);
    let entity1 = project.add_entity(basic);
    let conn1 = Connection::new(
        Relation {
            entity: entity1,
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
        Rect::new(pos1, Size::ZERO),
        Rect::new(pos2, Size::ZERO),
    );

    project.connect(conn1);
    
    let pos3 = (-15, -10);

    let conn2 = Connection::new(
        Relation {
            entity: entity1,
            relation: RelationType::Composition,
            multiplicity: Multiplicity::One,
        },
        Relation {
            entity: project.add_entity(Entity::new(
                "Idk2".to_string(),
                EntityType::AbstractClass,
                pos3,
            )),
            relation: RelationType::Association,
            multiplicity: Multiplicity::Many,
        },
        vec![],
        Rect::new(pos1, Size::ZERO),
        Rect::new(pos3, Size::ZERO),
    );

    project.connect(conn2);

    project
}
