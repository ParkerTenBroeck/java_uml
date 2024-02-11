package nodes;

public class BinaryNode implements Node{
    String data;
    BinaryNode left;
    BinaryNode right;

    public BinaryNode(String data, BinaryNode left, BinaryNode right){
        this.data = data;
        this.left = left;
        this.right = right;
    }

    public BinaryNode(String data){
        this.data = data;
        this.right = null;
        this.left = null;
    }

    @Override
    public boolean isLeaf() {
        return this.left == null && this.right == null;
    }

    @Override
    public boolean isLastChild() {
        throw new RuntimeException("bruh!");
    }

    public BinaryNode getLeft(){
        return this.left;
    }

    public BinaryNode getRight(){
        return this.right;
    }
    @Override
    public String getData() {
        return data;
    }

    @Override
    public Node[] getChildern() {
        return new Node[]{left, right};
    }
}
