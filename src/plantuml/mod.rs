use crate::java::{
    ast::{class::{Class, ClassType}, variable::Variable, Visibility},
    project::Project,
};

type Result = std::io::Result<()>;

pub struct PlantUmlGen<'a, T: std::io::Write> {
    java: &'a Project<'a>,
    out: &'a mut T,
}

impl<'a, T: std::io::Write> PlantUmlGen<'a, T> {
    pub fn new(out: &'a mut T, java: &'a Project<'a>) -> Self {
        Self { java, out }
    }

    pub fn write(&mut self) -> Result {
        writeln!(
            self.out,
            "@startuml
skinparam fixCircleLabelOverlapping true 
skinparam nodesep 100
skinparam ranksep 100
'skinparam linetype ortho
'skinparam linetype polyline
!pragma layout elk            
            "
        )?;

        for class in self.java.type_map.values() {
            self.write_class(class)?;
        }

        Ok(())
    }

    fn write_class(&mut self, class: &Class) -> Result {
        self.write_visibility(class.visibility)?;
        let kind = if class.modifiers.m_abstract() {
            "abstract class"
        } else {
            match &class.class_type {
                ClassType::Class => "class",
                ClassType::Interface => "interface",
                ClassType::Enum(_enums) => "enum",
                ClassType::Record => "class",
            }
        };
        self.out.write_all(kind.as_bytes())?;
        self.out.write_all(" ".as_bytes())?;   
        self.out.write_all(class.class_path.path.as_bytes())?;
        self.out.write_all(" ".as_bytes())?;    
        if let ClassType::Record = class.class_type{
            self.out.write_all("<<record>>".as_bytes())?;
        }
        if class.modifiers.m_static() && !matches!(class.class_type, ClassType::Enum(_) | ClassType::Interface) {
            self.out.write_all("<<static>>".as_bytes())?;
        }
        if class.modifiers.m_static() && !matches!(class.class_type, ClassType::Enum(_) | ClassType::Record){
            self.out.write_all("<<final>>".as_bytes())?;
        }

        self.out.write_all(" {\n  ".as_bytes())?;
        if let ClassType::Enum(enums) = &class.class_type {
            for name in enums {
                self.out.write_all("  ".as_bytes())?;
                self.out.write_all(name.as_bytes())?;
                self.out.write_all("\n".as_bytes())?;
            }
    
            self.out.write_all("  ==".as_bytes())?;
        }




        self.out.write_all("\n}\n".as_bytes())?;
        Ok(())
    }

    fn write_visibility(&mut self, vis: Visibility) -> Result {
        let buf = match vis {
            Visibility::Public => "+",
            Visibility::Protected => "#",
            Visibility::Private => "-",
            Visibility::None => "~",
        };
        self.out.write_all(buf.as_bytes())
    }

    fn visit_variable(&mut self, variable: &Variable) -> Result {
        Ok(())
    }
}
