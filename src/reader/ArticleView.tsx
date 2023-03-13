import { useEffect, useState } from "react";
import { Box, Heading, HStack } from "@chakra-ui/react";
import { IconHeadphones, IconLink, IconStar } from "@tabler/icons-react";
import { useStore } from "../lib/store";
import { fmtDatetime } from "../utils";
import { ArticleType } from "./types";

type ViewProps = {
  article: ArticleType | null;
  starArticle: (url: string, status: number) => Promise<void>;
};

export function ArticleView(props: ViewProps) {
  const { article, starArticle } = props;
  //const [isStar, setIsStar] = useState(article?.star_status === 1);
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
      // setIsStar(article.star_status === 1);
    }
  }, [article]);

  if (!article) {
    return (
      <div className=""></div>
    );
  }

  const { title, feed_url, channel_link, author, published } = article;

  return (
    <Box h="full">
      <div className="px-2 mb-1">
        <Heading className="m-1 text-3xl font-bold dark:text-white">{title}</Heading>
        <HStack className="flex items-center justify-start">
          <span className="m-1 dark:text-slate-400">{fmtDatetime(published || '')}</span>
          <span className="m-1 dark:text-slate-400">{author}</span>
          <a
            className="m-1 dark:text-slate-400"
            target="_blank"
            rel="noreferrer"
            href={feed_url}
          >
            <IconLink size={20} />
          </a>
          {/* <span 
            className="m-1 cursor-pointer" 
            onClick={async () => {
              // await starArticle(article.url, Math.abs(article.star_status - 1));
              // setIsStar(!isStar);
            }}
          >
            <IconStar size={20} className={`text-red-500 ${isStar ? 'fill-red-500' : ''}`} />
          </span> */}
          {article.audio_url.trim() && (
            <span 
              className="m-1 cursor-pointer" 
              onClick={() => setCurrentPod(
                {title, url: article.audio_url, published: new Date(article.published!), article_url: article.feed_url, channel_link: article.channel_link}
              )}
            >
              <IconHeadphones size={20} color="purple" />
            </span>
          )}
        </HStack>
      </div>
      <Box p={2}>
        <div
          className="content"
          color=""
          // eslint-disable-next-line react/no-danger
          dangerouslySetInnerHTML={{__html: pageContent}}
        />
      </Box>
    </Box>
  );
}
