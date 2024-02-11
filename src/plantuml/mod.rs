use crate::java::{ast::{class::Class, variable::Variable}, project::Project};


type Result = std::io::Result<()>;

pub struct PlantUmlGen<'a, T: std::io::Write>{
    java: &'a Project<'a>,
    out: &'a mut T
}

impl<'a, T: std::io::Write> PlantUmlGen<'a, T>{
    pub fn new(out: &'a mut T, java: &'a Project<'a>) -> Self{
        Self { java, out }
    }

    pub fn write(&mut self) -> Result{

        writeln!(self.out,         
"@startuml
skinparam fixCircleLabelOverlapping true 
skinparam nodesep 100
skinparam ranksep 100
'skinparam linetype ortho
'skinparam linetype polyline
!pragma layout elk            
            ")?;

            for class in self.java.type_map.values(){
                self.write_class(class)?;
            }

            Ok(())
    }

    fn write_class(&mut self, class: &Class) -> Result{

        Ok(())
    }

    fn visit_variable(&mut self, variable: &Variable) -> Result{

        Ok(())
    }
}