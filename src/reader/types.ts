export interface ChannelType {
  id: number;
  title: string;
  link: string;
  description?: string;
  published?: string; // iso date string
  ty: string; // podcast | rss
  unread: number;
}

export interface ArticleType {
  id: number;
  title: string;
  channel_link: string;
  feed_url: string;
  audio_url: string;
  intro: string;
  published?: number;
  content?: string;
  author?: string;
  img?: string;
}

export interface PodType {
  title: string;
  url: string;
  published?: Date;
  article_url: string;
  channel_link: string;
}