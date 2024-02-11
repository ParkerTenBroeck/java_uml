pub mod ast;
pub mod parser;
pub mod project;
pub mod tokenizer;
pub mod code_gen;

#[test]
fn test() {
    static TEST: &str = r#"
    package bruh;

    import java.util.ArrayList;
    import java.util.Comparator;
    import java.util.List;
    import java.util.function.Consumer;
    import java.util.function.Function;
    
 
    public class Test<T extends Object> implements Function {
        /* 
        public static <T extends Object & Consumer> void test2(List<? super Object> list) {
    
        }
    
        public static void test(ArrayList<? extends Function<? extends boolean[], Object>> test, ArrayList<? extends Comparator> test2){
    
        }
    
        public static void tes4t(ArrayList<? super Function<? extends boolean[], Object>> test, ArrayList<? extends Comparator> test2){
    
        }

    
        interface Bruh{
            short[][][][][][][][][][][][][][][][][][][][][][][][][][][][][][] bruh(int v1);
        }
    
        public static void bruh(){
            Bruh bruh = short[][][][][][][][][][][][][][][][][][][][][][][][][][][][][][]::new;
            bruh.bruh(69420);
        }
    
        public static <T extends Function<? extends boolean[], Object>> void tes5t(ArrayList<T> test, ArrayList<? extends Comparator> test2){
        }
    
        private static class Bruh_1{
            private static class Bruh_2{
    
            }
        }
    
        enum Test2{
            V1(){},
            V2{},
            Test2(){;;},
            V3(1,4,5,3,1,2,3,4,5){
            };{;}{}{}
            static{System.out.println("asdasdasd");};;;;;;;{}{};;;;;;;;;;;;;;;;;;{{;;;;;;;;;;;;;;;};}{{{{{{{{{{{{{{{{{{{{{}}}}}}}}}}}}}}}}}}}}}
            static {}
            Test2(){
            }
            ;
            Test2(int... v){
            }
            static Test2[] all(){
                return new Test2[]{Test2.V1, Test2.V2, Test2.Test2, Test2.V3};
            }
        }
    
        public sealed abstract class Service permits Car, Truck {
    
        }
    
        public non-sealed class Car<Y> extends Service{
    
        }
    
        public final class Truck extends Service{
    
        }
    
        public sealed interface Service2 permits Car2, Truck2 {
    
        }
    
        public static non-sealed class Car2<Y> implements Service2{
    
        }
    
        public static final class Truck2 implements Service2{
    
        }
    
        */

        @Override
        public Object apply(Object o) {
            return null;
        }

        record PostTags<T>(T postId, List<String> tags) {
            PostTags {
                tags: List.copyOf(tags);
            }
            public void stupid(Bruh... test){}
        }

        private static class Bruh_1{
            private static class Bruh_2{
    
            }
        }
    }
    "#;

    match parser::Parser::new(TEST).parse() {
        Ok(ok) => {
            println!("{:#?}", ok);
        }
        Err(err) => {
            println!("{:#?}", err);
            match err {
                parser::ParseError::ExpectedToken { range, .. }
                | parser::ParseError::UnexpectedToken { range, .. } => {
                    println!("{}", &TEST[..range.start]);
                }
                _ => {}
            }
        }
    }
}
pub fn test_project() {
    // use std::io::Write;

    let mut files = project::Files::new();
    files
        .load_dir("./test_java/p1")
        .unwrap();

    let mut project = project::Project::parse_all(&files).unwrap();

    project.resolve_imports();
    project.resolve_types();
    println!("{:#?}", project);
    std::thread::sleep(std::time::Duration::from_millis(1000));
}
