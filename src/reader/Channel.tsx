import { memo, useEffect, useState } from "react";
import { Flex, Text, Spinner, Tooltip, Box } from "@chakra-ui/react";
import { IconCircle, IconCircleCheck, IconRefresh } from "@tabler/icons-react";
import { fmtDatetime, dateCompare } from '../utils';
import { ArticleType, ChannelType } from "./types";

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
    channel, starChannel, articles, handleRefresh, updateAllReadStatus, onClickArticle, loading, syncing 
  } = props;

  if (loading) {
    return (
      <div className="flex items-center justify-center"><Spinner className="w-8 h-8" /></div>
    );
  } else if (!articles) {
    return (<></>);
  }

  return (
    <Flex className="flex flex-col items-between justify-center">
      <Flex className="flex flex-row items-center justify-between p-2 bg-slate-500 rounded">
        <Text className="font-bold">{channel?.title || (starChannel ? 'Starred' : '')}</Text>
        {(channel) && (
          <Flex className="flex flex-row items-center justify-end">
            <Tooltip label="Mark All Read" placement="bottom">
              <button className="" onClick={async () => await updateAllReadStatus(channel.link, 1)}>
                <IconCircleCheck size={18} className="m-1 dark:text-white" />
              </button>
            </Tooltip>
            <Tooltip label="Refresh Channel" placement="bottom">
              <button className="" onClick={handleRefresh}>
                <IconRefresh size={18} className="m-1 dark:text-white" />
              </button>
            </Tooltip>
          </Flex>
        )}
      </Flex>
      {syncing && <Spinner className="w-4 h-4" />}
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
        return n2.published && n1.published 
          ? dateCompare(n2.published, n1.published)
          : 0;
      })
      .sort((n1, n2) => n1.read_status - n2.read_status)
    : articles;

  // console.log("sorted: ", sortedArticles)

  return (
    <Box className="">
      {sortedArticles.map((article: ArticleType, idx: number) => {
        return (
          <ArticleItem
            key={`${article.id}=${idx}`}
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
  const [readStatus, setReadStatus] = useState(article.read_status);

  const handleClick = async () => {
    if (onArticleSelect) {
      await onArticleSelect(article);
      setReadStatus(1);
    }
  };

  useEffect(() => { setReadStatus(article.read_status); }, [article.read_status])

  const itemClass = `cursor-pointer flex flex-col items-start justify-center my-1 hover:bg-gray-200 dark:hover:bg-gray-800 ${highlight ? 'bg-blue-200 dark:bg-blue-800' : ''}`;

  return (
    <Flex
      className={itemClass}
      onClick={handleClick}
      aria-hidden="true"
    >
      <Flex className="flex flex-row items-center justify-start">
        {(readStatus === 0) && <IconCircle className="w-2 h-2 m-1 text-blue-500 fill-blue-500" />}
        <div className="flex-1 font-bold m-1 dark:text-white">{article.title}</div>
      </Flex>
      <Box className="flex flex-row items-center justify-center">
        <Text className="m-1 pl-2 text-sm dark:text-slate-400">
          {fmtDatetime(article.published || '')}
        </Text>
      </Box>
    </Flex>
  );
});
