import nodes.BinaryNode;
import nodes.LinkedNode;
import nodes.Node;
import visitors.BoringVisitor;
import visitors.DirVisitor;
import visitors.Visitor;

import java.util.LinkedList;

public class Main {

    @interface Test{
        
    }

    int[] intArr;
    int[] intArr2[];
    int intArr3[][];

    int test, value, multiple;
    int t1est, v1alue, m1ultiple = 34;
    int[] arr1, arr2[], arr3[][] = null;

    public void bruhbruh(@Deprecated double[] v[]){

    }

    public static void main(String... args) throws Exception {


        {
            var t = new BinarySearchTree();
            var vals = new int[]{1,0,8,6,2,3,4,5,9};
            for(int v:vals){
                t.insert(v);
            }
            t.print();
            System.out.println("\n\n");
            vals = t.toArr();
            for(int v:vals){
                System.out.print(v + " ");
            }
            System.out.println();
        }

        {
            var visitor = new BoringVisitor();
            BinaryNode tree;
            {
                var d = new BinaryNode("d");
                var e = new BinaryNode("e");
                var g = new BinaryNode("g", null, new BinaryNode("h"));
                var f = new BinaryNode("f", g, null);
                var c = new BinaryNode("c", e, f);
                var b = new BinaryNode("b", d, null);
                var a = new BinaryNode("a", b, c);
                tree = a;
            }

            System.out.print("breath first: ");
            breathfirst(visitor, tree);
            System.out.println();
            System.out.print("preorder      ");
            preorder(visitor, tree);
            System.out.println();
            System.out.print("preorder3     ");
            preorder3(visitor, tree);
            System.out.println();
            System.out.print("post order    ");
            postorder(visitor, tree);
            System.out.println();
            System.out.print("inorder       ");
            inorder(visitor, tree);
            System.out.println();
        }

        {
            var visitor = new DirVisitor();
            var tree = new LinkedNode("My Book",
                    new LinkedNode("Ch1",
                            new LinkedNode("Section 1",
                                    new LinkedNode("This is text stored in this silly book :)")),
                            new LinkedNode("Section 2",
                                    new LinkedNode(":)"),
                                    new LinkedNode(">.<"))),
                    new LinkedNode("Ch2",
                            new LinkedNode("Section 3"), new LinkedNode("Section 4"), new LinkedNode("Section 5")),
                    new LinkedNode("Ch3",
                            new LinkedNode("Section 6")),
                    new LinkedNode("Ch4"));
            preorder2(visitor, tree);
        }
    }

    public static void breathfirst(Visitor visitor, Node node){
        LinkedList<Node> nodes = new LinkedList<>();
        nodes.addFirst(node);

        while(nodes.size() > 0){
            var tmp = nodes.removeFirst();
            for(Node child : tmp.getChildern()){
                if (child == null)
                    continue;

                nodes.addLast(child);
            }
            visitor.visitNode(tmp);
        }
    }

    public static void preorder(Visitor visitor, Node node){
        visitor.visitNode(node);
        visitor.down();
        for (Node child: node.getChildern()){
            if (child != null)
                preorder(visitor, child);
        }
        visitor.up();
    }

    public static void preorder3(Visitor visitor, BinaryNode node){
        LinkedList<BinaryNode> rightNodes = new LinkedList<>();
        rightNodes.add(node);

        while(rightNodes.size() != 0){
            node = rightNodes.removeFirst();
            while(node != null){
                visitor.visitNode(node);
                if (node.getRight() != null){
                    rightNodes.addLast(node.getRight());
                }
                node = node.getLeft();
            }
        }
    }

    public static void postorder(Visitor visitor, Node node){
        visitor.down();
        for (Node child: node.getChildern()){
            if (child != null)
                postorder(visitor, child);
        }
        visitor.up();

        visitor.visitNode(node);
    }

    public static void inorder(Visitor visitor, BinaryNode node){
        if(node==null)
            return;

        visitor.down();
        inorder(visitor, node.getLeft());
        visitor.up();

        visitor.visitNode(node);

        visitor.down();
        inorder(visitor, node.getRight());
        visitor.up();
    }



    public static void preorder2(Visitor visitor, LinkedNode node){
        visitor.visitNode(node);
        var child = node.getFirstChild();
        visitor.down();
        while(child != null){
            preorder2(visitor, child);
            child = child.getNextSibling();
        }
        visitor.up();
    }

    public enum BRUH{
        V1
    }
    public enum BRUH2{
        V1,
    }
    public enum BRUH3{
        V1;
    }
    public enum BRUH4{
    }
}