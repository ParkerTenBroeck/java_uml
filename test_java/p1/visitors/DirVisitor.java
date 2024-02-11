package visitors;

import nodes.Node;

public class DirVisitor implements Visitor{
    int level = 0;

    boolean last_prev_node_last_child;
    String start = "";

    @Override
    public void visitNode(Node node) {
        System.out.print(this.start);
        if (level != 0){
            if (node.isLastChild()){
                System.out.print("┗");
            }else{
                System.out.print("┣");
            }
        }
        System.out.println(node.getData());
        this.last_prev_node_last_child = node.isLastChild();
    }

    @Override
    public void down() {
        if (level != 0){
            if(this.last_prev_node_last_child){
                start = start + " ";
            }else{
                start = start + "┃";
            }
        }
        level ++;
    }

    @Override
    public void up() {
        level --;
        if (level == 0){
            start = "";
        }else{
            start = start.substring(0, start.length() - 1);
        }
    }
}
