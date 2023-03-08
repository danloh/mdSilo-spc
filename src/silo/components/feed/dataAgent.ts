import { ChannelType, ArticleType } from "../../types/model";

type FeedResult = {
  channel: ChannelType;
  articles: ArticleType[];
};

export const fetchFeed = async (url: string): Promise<FeedResult | null> => {
  return null
}

export const addChannel = async (
  url: string, ty: string, title: string | null
): Promise<number> => {
  return 0
}

export const importChannels = async (list: string[]) => {
  return 
}

export const getChannels = async (): Promise<ChannelType[]> => {
  return []
}

export const deleteChannel = async (link: string) => {
  return 
};

export const getArticleList = async (
  feedLink: string | null, 
  readStatus: number | null,
  starStatus: number | null,
) : Promise<ArticleType[]> => {
  return []
}

export const getArticleByUrl = async (url: string): Promise<ArticleType | null> => {
  return null
}

export const getUnreadNum = async (): Promise<{ [key: string]: number }> => {
  return {jj: 0}
}

export const updateArticleStarStatus = async (
  articleUrl: string, 
  star_status: number, // 0 || 1
): Promise<number> => {
  return 0
}

export const updateArticleReadStatus = async (
  articleUrl: string, 
  read_status: number,
): Promise<number> => {
  return 0
}

export const updateAllReadStatus = async (
  feedLink: string, 
  readStatus: number,
): Promise<number> => {
  return 0 //await invoke('update_all_read_status', { feedLink, readStatus })
}
