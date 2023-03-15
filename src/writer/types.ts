export interface NoteType {
  id: string;
  title: string;
  content: string;
  uname: string;
  folder: string;
  created_at: number;
  updated_at: number;
}

export interface SimpleNote {
  id: string;
  title: string;
  uname: string;
  folder: string;
  created_at: number;
  updated_at: number;
}
