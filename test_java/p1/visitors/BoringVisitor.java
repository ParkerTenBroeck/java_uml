package visitors;

import nodes.Node;

public class BoringVisitor implements Visitor{
    @Override
    public void visitNode(Node node) {
        System.out.print(node.getData());
    }

    @Override
    public void down() {

    }

    @Override
    public void up() {

    }
}
