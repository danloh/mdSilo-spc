import { memo, useEffect, useState } from "react";
import { Flex, Text, Spinner, Tooltip, Box, HStack } from "@chakra-ui/react";
import { IconCircleCheck, IconRefresh } from "@tabler/icons-react";
import { fmtDatetime } from '../utils';
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
    return (<Spinner />);
  } else if (!articles) {
    return (<></>);
  }

  return (
    <Flex direction="column" p={2} className="items-between justify-center">
      <HStack p={2} className="bg-slate-500 rounded">
        <Text as="b" fontSize="xl">{channel?.title || (starChannel ? 'Starred' : '')}</Text>
        {(channel) && (
          <>
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
          </>
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

  // console.log("sorted: ", sortedArticles)

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
  //const [readStatus, setReadStatus] = useState(article.read_status);

  const handleClick = async () => {
    if (onArticleSelect) {
      await onArticleSelect(article);
      //setReadStatus(1);
    }
  };

  //useEffect(() => { setReadStatus(article.read_status); }, [article.read_status])

  return (
    <Flex
      direction="column"
      cursor="pointer"
      my={1}
      bgColor={highlight ? 'blue.200' : ''}
      _hover={{bgColor: "gray.200"}}
      onClick={handleClick}
      aria-hidden="true"
    >
      <Text as="b" m={1} className="font-bold dark:text-white">{article.title}</Text>
      <Text as="i" fontSize="sm" m={1} className="dark:text-slate-400">
        {fmtDatetime(article.published || '')}
      </Text>
    </Flex>
  );
});
