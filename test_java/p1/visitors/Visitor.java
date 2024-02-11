package visitors;

import nodes.Node;

public interface Visitor {
    void visitNode(Node node);
    void down();
    void up();
}
