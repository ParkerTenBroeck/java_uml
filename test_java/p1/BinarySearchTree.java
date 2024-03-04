import nodes.BinaryNode;
import nodes.Node;

import java.util.LinkedList;

public class BinarySearchTree {

    Node root;
    int size;


    public void insert(int val) {
        if (this.root == null){
            this.root = new Node(val);
        }else{
            var node = root;
            while(true){
                if (val < node.val){
                    if (node.left == null){
                        node.left = new Node(val);
                        break;
                    }else{
                        node = node.left;
                    }
                }else if (node.val < val){
                    if (node.right == null){
                        node.right = new Node(val);
                        break;
                    }else{
                        node = node.right;
                    }
                }else{
                    throw new RuntimeException("equalll!!!!" + node.val + " " + val);
                }
            }
        }
        size ++;
    }

    private static void insert(final Node node, int val){

    }

    public void has(int val){

    }

    public void print(){
        print(root);
    }
    private void print(Node node){
        if (node.left != null)
            print(node.left);

        System.out.println(node.val + " ");

        if (node.right != null)
            print(node.right);
    }

    public int[] toArr(){
        int[] arr = new int[this.size];
        LinkedList<Node> rightNodes = new LinkedList<>();
        rightNodes.add(this.root);

        int i = 0;
        while(rightNodes.size() != 0){
            var node = rightNodes.removeFirst();
            while(node != null){
                arr[i] = node.val;
                i ++;
                if (node.right != null){
                    rightNodes.addLast(node.right);
                }
                node = node.left;
            }
        }
        return arr;
    }


    private class Node{
        int val;
        Node left;
        Node right;

        public Node(int val){
            this.val = val;
        }

        public Node(int val, Node left, Node right){
            this.val = val;
            this.left = left;
            this.right = right;
        }
    }
}
