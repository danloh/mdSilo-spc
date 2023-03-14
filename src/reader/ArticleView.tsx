import { useEffect, useState } from "react";
import { Box, Button, Heading, HStack, Text } from "@chakra-ui/react";
import { TbHeadphones, TbLink, TbStar } from "react-icons/tb";
import { useStore } from "../lib/store";
import { fmtDatetime } from "../utils";
import { ArticleType, PodType } from "./types";
import * as dataAgent from '../dataAgent';

type ViewProps = {
  article: ArticleType | null;
  starArticle: (url: string, status: number) => Promise<void>;
};

export function ArticleView(props: ViewProps) {
  const { article, starArticle } = props;
  const [isStar, setIsStar] = useState(false);
  const [pageContent, setPageContent] = useState("");

  const setCurrentPod = useStore(state => state.setCurrentPod);

  useEffect(() => {
    if (article) {
      const content = (article.content || article.intro || "").replace(
        /<a[^>]+>/gi,
        (a: string) => {
          return (!/\starget\s*=/gi.test(a)) ? a.replace(/^<a\s/, '<a target="_blank"') : a;
        }
      );
      setPageContent(content);
      dataAgent.checkArticleStarStatus(article.feed_url).then(res => {
        setIsStar(res);
      })
    }
  }, [article]);

  if (!article) {
    return (
      <div className=""></div>
    );
  }

  const { title, feed_url, author, published } = article;

  return (
    <Box h="full">
      <div className="px-2 mb-1">
        <Heading className="m-1 text-3xl font-bold dark:text-white">{title}</Heading>
        <HStack className="flex items-center justify-start">
          <Text className="m-1 dark:text-slate-400">{fmtDatetime(published || '')}</Text>
          <Text className="m-1 dark:text-slate-400">{author}</Text>
          <a
            className="m-1 dark:text-slate-400"
            target="_blank"
            rel="noreferrer"
            href={feed_url}
          >
            <TbLink size={20} />
          </a>
          <Button 
            size="xs" 
            title={`${isStar ? 'Starred' : 'Star'}`}
            onClick={async () => {
              await starArticle(article.feed_url, Math.abs(Number(isStar) - 1));
              setIsStar(!isStar);
            }}
          >
            <TbStar size={20} color={`${isStar ? 'red' : 'green'}`} />
          </Button>
          {article.audio_url.trim() && (
            <Button 
              size="xs" 
              onClick={() => setCurrentPod(articleToPod(article))}
            >
              <TbHeadphones size={20} color="purple" />
            </Button>
          )}
        </HStack>
      </div>
      <Box p={2}>
        <div
          className="content"
          // eslint-disable-next-line react/no-danger
          dangerouslySetInnerHTML={{__html: pageContent}}
        />
      </Box>
    </Box>
  );
}

export function articleToPod (article: ArticleType): PodType {
  return {
    title: article.title, 
    url: article.audio_url, 
    published: new Date(article.published! * 1000), 
    article_url: article.feed_url, 
    channel_link: article.channel_link,
  };
}
