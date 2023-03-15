import { ChannelType, ArticleType } from "./reader/types";
import { NoteType, SimpleNote } from "./writer/types";

export const postReq = async (url: string, data: any): Promise<any> => {
  let options = {
    method:  'POST', 
    headers: {'Content-Type': 'application/json'},
    body: JSON.stringify(data)
  };
  let resp = await fetch(url, options);
  return await resp.json();
}

// ==========================================
// ==================== feed reader =========
// ==========================================

type FeedResult = {
  channel: ChannelType;
  articles: ArticleType[];
};

export const fetchFeed = async (url: string): Promise<FeedResult> => {
  let resp = await fetch(`/api/fetch_feed?url=${url}`);
  // if (!resp.ok) return;
  return await resp.json();
}

export const addChannel = async (
  url: string, ty: string | null, title: string | null
): Promise<number> => {
  return await postReq(`/api/add_channel`, {url, title, ty});
}

export const getChannels = async (): Promise<ChannelType[]> => {
  let resp = await fetch(`/api/get_channels`);
  return await resp.json();
}

export const deleteChannel = async (link: string) => {
  return //await invoke('delete_channel', { link })
};

export const getArticleList = async (url: string) : Promise<ArticleType[]> => {
  let resp = await fetch(`/api/get_channel_feeds?url=${url}`);
  return await resp.json();
}

export const getAudioArticles = async () : Promise<ArticleType[]> => {
  let resp = await fetch(`/api/get_audio_feeds`);
  return await resp.json();
}

export const getStarArticles = async () : Promise<ArticleType[]> => {
  let resp = await fetch(`/api/get_star_feeds`);
  return await resp.json();
}

export const getReadArticles = async () : Promise<ArticleType[]> => {
  let resp = await fetch(`/api/get_read_feeds`);
  return await resp.json();
}

export const getArticleByUrl = async (url: string): Promise<ArticleType | null> => {
  return null //await invoke('get_article_by_url', { url })
}

// TODO
export const getUnreadNum = async (): Promise<{ [key: string]: number }> => {
  return {} //await invoke('get_unread_num')
}

export const updateArticleStarStatus = async (
  url: string, 
  star_status: number, // 0 | 1,
): Promise<number> => {
  let resp = star_status 
    ? await fetch(`/api/star_feed?url=${url}`)
    : await fetch(`/api/unstar_feed?url=${url}`);
  return resp.ok ? 1 : 0;
}

export const updateArticleReadStatus = async (url: string): Promise<number> => {
  let resp = await fetch(`/api/read_feed?url=${url}`);
  return resp.ok ? 1 : 0;
}

export const checkArticleStarStatus = async (url: string): Promise<boolean> => {
  let resp = await fetch(`/api/check_star?url=${url}`);
  if (resp.ok) {
    return await resp.json();
  } else {
    return false;
  }
}

export const checkArticleReadStatus = async (url: string): Promise<boolean> => {
  let resp = await fetch(`/api/check_read?url=${url}`);
  if (resp.ok) {
    return await resp.json();
  } else {
    return false;
  }
}

export const updateAllReadStatus = async (
  feedLink: string, 
  readStatus: number,
): Promise<number> => {
  return 0 // await invoke('update_all_read_status', { feedLink, readStatus })
}


// ==========================================
// ==================== note ================
// ==========================================

export const newNote = async (
  id: string, title: string, content: string
): Promise<NoteType> => {
  return await postReq(`/api/new_note`, {id, title, content});
}

export const renameNote = async (
  id: string, title: string
): Promise<NoteType> => {
  return await postReq(`/api/rename_note`, {id, title, content: ''});
}

export const moveNote = async (
  id: string, folder: string
): Promise<NoteType> => {
  let resp = await fetch(`/api/move_note/${id}/${folder}`);
  return await resp.json();
}

export const delNote = async (id: string): Promise<NoteType> => {
  let resp = await fetch(`/api/del_note/${id}`);
  return await resp.json();
}

export const getNote = async (id: string): Promise<NoteType> => {
  let resp = await fetch(`/api/get_note/${id}`);
  return await resp.json();
}

export const getNotesByFolder = async (
  folder: string
): Promise<[SimpleNote[], number]> => {
  let resp = await fetch(`/api/get_folder_notes/${folder}`);
  return await resp.json();
}

export const getFolders = async (): Promise<string[]> => {
  let resp = await fetch(`/api/get_folders`);
  return await resp.json();
}
