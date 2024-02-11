package nodes;

public interface Node {
    public String getData();

    public Node[] getChildern();

    boolean isLeaf();

    boolean isLastChild();
}
