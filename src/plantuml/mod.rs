use crate::java::{
    ast::{class::{Class, ClassType}, functions::{self, Function, FunctionKind}, generics::{GenericInvoctionPart, WildcardBound}, types::{JType, Primitive, TypePath, TypeResolution}, variable::Variable, JPath, Visibility},
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
        self.out.write_all("@startuml
        skinparam fixCircleLabelOverlapping true 
        skinparam nodesep 100
        skinparam ranksep 100
        'skinparam linetype ortho
        'skinparam linetype polyline
        !pragma layout elk      
        set separator ::      
                    ".as_bytes())?;

        for class in self.java.type_map.values() {
            self.write_class(class)?;
        }



        self.out.write_all("@enduml".as_bytes())
    }

    fn write_full_class_path(&mut self, class_path: &JPath, package: &Option<JPath>) -> Result{
        let remainder = if let Some(package) = package{
            for part in package.path.split('.'){
                self.out.write_all(part.as_bytes())?;
                self.out.write_all("::".as_bytes())?;    
            }
            class_path.path.trim_start_matches(&package.path)
        }else{
            class_path.path.as_str()
        };
        { 
            let mut peek = remainder.split('.').peekable();
            while let Some(part) = peek.next(){
                if part.is_empty(){
                    continue;
                }
                self.out.write_all(part.as_bytes())?;
                if peek.peek().is_some() {
                    self.out.write_all(".".as_bytes())?; 
                }   
            }
        }
        Ok(())
    }

    fn write_class_type_name(&mut self, jtype: &TypePath) -> Result{
        match &jtype.resolved{
            TypeResolution::Some(full_path) => {
                if let Some(some) = self.java.type_map.get(full_path){
                    let str = if let Some(package) = &some.package{
                        jtype.origional.path.trim_end_matches(&package.path)
                    }else{
                        &jtype.origional.path
                    };
                    self.out.write_all(str.as_bytes())
                }else{
                    self.out.write_all(jtype.origional.path.as_bytes())
                }
            },
            TypeResolution::None |
            TypeResolution::Generic => {
                self.out.write_all(jtype.origional.path.as_bytes())
            },
        }
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

        self.write_full_class_path(&class.class_path, &class.package)?;

        self.out.write_all(" ".as_bytes())?;    
        if let ClassType::Record = class.class_type{
            self.out.write_all("<<record>>".as_bytes())?;
        }
        if class.modifiers.m_static() && !matches!(class.class_type, ClassType::Enum(_) | ClassType::Interface) {
            self.out.write_all("<<static>>".as_bytes())?;
        }
        if class.modifiers.m_final() && !matches!(class.class_type, ClassType::Enum(_) | ClassType::Record){
            self.out.write_all("<<final>>".as_bytes())?;
        }

        self.out.write_all(" {\n".as_bytes())?;
        if let ClassType::Enum(enums) = &class.class_type {
            for name in enums {
                self.out.write_all("  ".as_bytes())?;
                self.out.write_all(name.as_bytes())?;
                self.out.write_all("\n".as_bytes())?;
            }
    
            self.out.write_all("  ==\n".as_bytes())?;
        }

        for variable in &class.variables{
            self.visit_variable(variable)?;
        }

        for function in &class.functions{
            self.visit_function(function)?;
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
        self.out.write_all("  ".as_bytes())?;
        if variable.modifiers.m_static() {
            self.out.write_all("{static} ".as_bytes())?;
        }
        self.write_visibility(variable.visibility)?;
        if variable.modifiers.m_final() {
            self.out.write_all("final ".as_bytes())?;
        }
        self.out.write_all(variable.name.as_bytes())?;
        self.out.write_all(": ".as_bytes())?;
        self.visit_type(&variable.jtype)?;
        
        self.out.write_all("\n".as_bytes())
    }
    
    fn visit_function(&mut self, function: &Function) -> Result {
        self.out.write_all("  ".as_bytes())?;

        if function.modifiers.m_static() {
            self.out.write_all("{static} ".as_bytes())?;
        }
        if function.modifiers.m_abstract() {
            self.out.write_all("{abstract} ".as_bytes())?;
        }

        self.write_visibility(function.visibility)?;

        if function.modifiers.m_final() {
            self.out.write_all("final ".as_bytes())?;
        }
        if function.modifiers.m_default() {
            self.out.write_all("default ".as_bytes())?;
        }

        self.out.write_all(function.name.as_bytes())?;

        if let Some(generics) = &function.generics{
            self.out.write_all(" <".as_bytes())?;
            for (index, gen) in generics.definitions.iter().enumerate(){
                
                self.out.write_all(gen.name.as_bytes())?;

                if let Some(bounds) = &gen.extend_bound{
                    self.out.write_all(" extends ".as_bytes())?;
                    self.write_type_list(bounds)?;
                }
                if index != generics.definitions.len() - 1{
                    self.out.write_all(", ".as_bytes())?;
                }
            }
            self.out.write_all("> ".as_bytes())?;

        }

        self.out.write_all("(".as_bytes())?;
        for (index, param) in function.parameters.iter().enumerate(){
            match param{
                functions::Parameter::Regular(jtype, name) => {
                    self.visit_type(jtype)?;
                    self.out.write_all(" ".as_bytes())?;
                    self.out.write_all(name.as_bytes())?;
                },
                functions::Parameter::VArgs(jtype, name) => {
                    self.visit_type(jtype)?;
                    self.out.write_all("... ".as_bytes())?;
                    self.out.write_all(name.as_bytes())?;

                },
            }
            if index != function.parameters.len() - 1{
                self.out.write_all(", ".as_bytes())?;
            }
        }
        self.out.write_all(")".as_bytes())?;

        match &function.kind{
            FunctionKind::Regular(ret) => {
                self.out.write_all(": ".as_bytes())?;
                self.visit_type(ret)?;
            },
            FunctionKind::Constructor
            | FunctionKind::CompactConstructor => {},
        }

        if let Some(throws) = &function.throws{
            self.out.write_all(" throws ".as_bytes())?;
            self.write_type_list(throws)?;
        }
        self.out.write_all("\n".as_bytes())
    }

    fn visit_type(&mut self, jtype: &JType) -> Result{
        match jtype{
            JType::Primitive(prim) => self.write_primitive(prim),
            JType::PrimitiveArr(prim, arr) => {
                self.write_primitive(prim)?;
                for _ in 0..arr.get(){
                    self.out.write_all("[]".as_bytes())?;
                }
                Ok(())
            },
            JType::Object { path, generics, arr } => {

                self.write_class_type_name(path)?;
                if let Some(gen) = generics{
                    self.out.write_all("<".as_bytes())?;
                    for (index, inv) in gen.invoctions.iter().enumerate(){
                        match inv{
                            GenericInvoctionPart::Type(jtype) => self.visit_type(jtype)?,
                            GenericInvoctionPart::Wildcard(kind) => {
                                self.out.write_all("?".as_bytes())?;

                                match kind{
                                    WildcardBound::None => {},
                                    WildcardBound::Extends(_) => {
                                        self.out.write_all(" extends ".as_bytes())?;
                                    },
                                    WildcardBound::Super(_) => {
                                        self.out.write_all(" super ".as_bytes())?;
                                    },
                                }
                                match kind{
                                    WildcardBound::None => {},
                                    WildcardBound::Extends(types) 
                                    | WildcardBound::Super(types) => {
                                        self.write_type_list(types)?;
                                    },
                                }
                            },
                        }
                        if index != gen.invoctions.len() - 1{
                            self.out.write_all(", ".as_bytes())?;
                        }
                    }
                    self.out.write_all(">".as_bytes())?;
                }

                for _ in 0..arr.map(|v|v.get()).unwrap_or(0){
                    self.out.write_all("[]".as_bytes())?;
                }
                Ok(())
            },
        }
    }

    fn write_type_list(&mut self, list: &[JType]) -> Result{
        for (index, jtype) in list.iter().enumerate(){
            self.visit_type(jtype)?;
            if index != list.len() - 1{
                self.out.write_all(", ".as_bytes())?;
            }
        }
        Ok(())
    }

    fn write_primitive(&mut self, prim: &Primitive) -> Result{
        use crate::java::ast::types::Primitive::*;
        let str = match prim{
            Byte => "byte",
            Short => "short",
            Int => "int",
            Long => "long",
            Float => "float",
            Double => "double",
            Char => "char",
            Void => "void",
            Boolean => "boolean",
        };
        self.out.write_all(str.as_bytes())
    }
}
