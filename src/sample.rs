use crate::data::entity::{Field, Method};
use crate::data::{
    connection::{Multiplicity, Relation, RelationType},
    entity::{AccessModifier, EntityType},
    Connection, Entity, Project,
};
use crate::geometry::{Rect, Size};

pub fn project() -> Project {
    let mut project = Project::new("Test".to_string());

    let pos1 = (-5, 20);
    let mut basic = Entity::new("Basic".to_string(), EntityType::Class, pos1);

    basic.fields.push(Field {
        name: "count".to_string(),
        r#type: "int".to_string(),
        modifier: AccessModifier::Public,
    });
    basic.fields.push(Field {
        name: "name".to_string(),
        r#type: "string".to_string(),
        modifier: AccessModifier::Private,
    });

    basic.methods.push(Method {
        name: "toString".to_string(),
        return_type: "string".to_string(),
        modifier: AccessModifier::Public,
        arguments: vec![],
    });
    
    basic.methods.push(Method {
        name: "increment".to_string(),
        return_type: "void".to_string(),
        modifier: AccessModifier::Public,
        arguments: vec!["amount".to_string()],
    });
    

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
