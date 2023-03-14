import { memo, useEffect, useState } from "react";
import { Flex, Text, Spinner, Tooltip, Box, HStack } from "@chakra-ui/react";
import { TbRefresh } from "react-icons/tb";
import { fmtDatetime, isUrl } from '../utils';
import { ArticleType, ChannelType } from "./types";
import * as dataAgent from '../dataAgent';
// import { useStore } from "../lib/store";
// import { articleToPod } from "./ArticleView";

type Props = {
  channel: ChannelType | null;
  starChannel?: boolean;
  articles: ArticleType[] | null;
  handleRefresh: () => Promise<void>;
  updateAllReadStatus: (feedLink: string, status: number) => Promise<void>;
  onClickArticle: (article: ArticleType) => Promise<void>;
  loading: boolean;
  syncing: boolean;
};

export function Channel(props: Props) {
  const { 
    channel, starChannel, articles, handleRefresh, onClickArticle, loading, syncing 
  } = props;

  if (loading) {
    return (<Spinner />);
  } else if (!articles) {
    return (<></>);
  }

  return (
    <Flex direction="column" p={2} className="items-between justify-center">
      <HStack p={2} className="bg-slate-500 rounded">
        <Text as="b" fontSize="xl">
          {channel?.title || (starChannel ? 'Starred' : 'Playlist')}
        </Text>
        {(channel) && (
          <Tooltip label="Refresh Channel" placement="bottom">
            <button className="" onClick={handleRefresh}>
              <TbRefresh size={18} className="m-1 dark:text-white" />
            </button>
          </Tooltip>
        )}
      </HStack>
      {syncing && <Spinner boxSize={6} />}
      <ArticleList
        articles={articles}
        onClickArticle={onClickArticle}
      />
    </Flex>
  );
}

type ListProps = {
  articles: ArticleType[];
  onClickArticle: (article: ArticleType) => Promise<void>;
};

function ArticleList(props: ListProps) {
  const { articles, onClickArticle } = props;
  const [highlighted, setHighlighted] = useState<ArticleType>();

  const onArticleSelect = async (article: ArticleType) => {
    setHighlighted(article);
    await onClickArticle(article);
  };

  const sortedArticles = articles.length >= 2 
    ? articles.sort((n1, n2) => {
        return (n2.published || 0) - (n1.published || 0);
      })
    : articles;

  return (
    <Box className="">
      {sortedArticles.map((article: ArticleType, idx: number) => {
        return (
          <ArticleItem
            key={`${article.id}-${idx}`}
            article={article}
            highlight={highlighted?.id === article.id}
            onArticleSelect={onArticleSelect}
          />
        )}
      )}
    </Box>
  );
}

type ItemProps = {
  article: ArticleType;
  onArticleSelect: (article: ArticleType) => Promise<void>;
  highlight: boolean;
};

const ArticleItem = memo(function ArticleItm(props: ItemProps) {
  const { article, onArticleSelect, highlight } = props;
  const [readStatus, setReadStatus] = useState(false);
  // const setCurrentPod = useStore(state => state.setCurrentPod);

  const handleClick = async () => {
    if (onArticleSelect) {
      await onArticleSelect(article);
      setReadStatus(true);
      // if (isUrl(article.audio_url.trim())) {
      //   setCurrentPod(articleToPod(article))
      // }
    }
  };

  useEffect(() => { 
    dataAgent.checkArticleReadStatus(article.feed_url).then(res => {
      setReadStatus(res);
    })
  }, [])

  return (
    <Flex
      direction="column"
      cursor="pointer"
      my={1}
      bgColor={highlight ? 'blue.200' : `${readStatus ? '' : 'blue.100'}`}
      _hover={{bgColor: "gray.200"}}
      onClick={handleClick}
    >
      <Text as="b" m={1} className="font-bold dark:text-white">{article.title}</Text>
      <Text as="i" fontSize="sm" m={1} className="dark:text-slate-400">
        {fmtDatetime(article.published || '')}
      </Text>
    </Flex>
  );
});
