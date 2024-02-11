use crate::java::project::Project;

fn test(){

}

pub fn generate_plantuml(out: &mut impl std::io::Write, java: &Project) -> std::io::Result<()>{

    writeln!(out,         
"@startuml
skinparam fixCircleLabelOverlapping true 
skinparam nodesep 100
skinparam ranksep 100
'skinparam linetype ortho
'skinparam linetype polyline
!pragma layout elk

")?;



    Ok(())
}