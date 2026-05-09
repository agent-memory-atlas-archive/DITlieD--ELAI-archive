export interface TreeNode {
  id: string;
  label: string;
  status?: 'done' | 'active' | 'pending' | 'failed';
  children?: TreeNode[];
  meta?: string;
}
