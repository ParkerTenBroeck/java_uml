package nodes;

public class LinkedNode implements Node{
    String data;
    LinkedNode firstChild;
    LinkedNode nextSibling;

    public LinkedNode(String data){
        this.data = data;
        this.firstChild = null;
        this.nextSibling = null;
    }

    public LinkedNode(String data, LinkedNode... children){
        this.data = data;
        if (children.length == 0){
            this.firstChild = null;
            this.nextSibling = null;
        }else{
            this.firstChild = children[0];
            for(int i = 1; i < children.length; i ++){
                children[i-1].nextSibling = children[i];
            }
        }
    }



    public LinkedNode getFirstChild(){
        return this.firstChild;
    }

    public LinkedNode getNextSibling() {
        return this.nextSibling;
    }

    @Override
    public boolean isLeaf() {
        return this.firstChild == null;
    }

    @Override
    public boolean isLastChild() {
        return this.nextSibling == null;
    }

    @Override
    public String getData() {
        return this.data;
    }

    @Override
    public Node[] getChildern() {
        return new Node[0];
    }
}
