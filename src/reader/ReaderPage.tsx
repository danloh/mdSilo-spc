import { useEffect, useState } from 'react';
import { Box, Flex } from '@chakra-ui/react';
import Split from "react-split";
import { store, useStore } from '../lib/store';
import { ArticleType, ChannelType } from './types';
import ErrorBoundary from '../misc/ErrorBoundary';
import { ChannelList } from './ChannelList';
import { Channel } from './Channel';
import { ArticleView } from './ArticleView';
import { FeedManager } from './FeedManager';
import * as dataAgent from '../dataAgent';
import AudioPlayer from './AudioPlayer';

export default function Feed() {
  // channel list
  const [channelList, setChannelList] = useState<ChannelType[]>([]);
  const [currentChannel, setCurrentChannel] = useState<ChannelType | null>(null);
  const [currentArticles, setCurrentArticles] = useState<ArticleType[] | null>(null);
  const [currentArticle, setCurrentArticle] = useState<ArticleType | null>(null);
  const [starChannel, setStarChannel] = useState(false);
  const [showManager, setShowManager] = useState(false);

  const storeArticle = useStore(state => state.currentArticle);
  const currentPod = useStore((state) => state.currentPod);

  const getList = () => {
    Promise.all(
      [dataAgent.getChannels(), dataAgent.getUnreadNum()]
    ).then(([channels, unreadNum]) => {
      channels.forEach((item) => {
        item.unread = unreadNum[item.link] || 0;
      });

      setChannelList(channels);
    })
  };

  useEffect(() => { getList(); }, []);

  const [refreshing, setRefreshing] = useState(false);
  const [doneNum, setDoneNum] = useState(0);
  const refreshChannel = async (link: string, ty: string, title: string) => {
    const res = await dataAgent.addChannel(link, ty, title);
    return res;
  };

  const refreshList = async () => {
    setRefreshing(true);
    setDoneNum(0);
    for (const channel of channelList) {
      await refreshChannel(channel.link, channel.ty, channel.title);
      setDoneNum(doneNum + 1);
    }
    setRefreshing(false);
  };

  const onShowManager = () => {
    setShowManager(!showManager);
  };

  const loadArticleList = async (link: string) => {
    const articles = await dataAgent.getArticleList(link);
    // console.log("current articles", articles, currentArticles);
    setCurrentArticles(articles);
  };

  const [loading, setLoading] = useState(false);
  const onClickFeed = async (link: string) => {
    setLoading(true);
    const clickedChannel = channelList.find(c => c.link === link);
    if (clickedChannel) {
      setCurrentChannel(clickedChannel);
      setShowManager(false);
      await loadArticleList(clickedChannel.link);
    } 
    setLoading(false);
  };

  const onBeforeClick = () => {
    setLoading(true);
    setCurrentChannel(null);
    setCurrentArticles(null);
    setShowManager(false);
  };

  const onClickStar = async () => {
    onBeforeClick();
    setStarChannel(true);
    const starArticles = await dataAgent.getStarArticles();
    setCurrentArticles(starArticles);
    setLoading(false);
  };

  const onClickAudio = async () => {
    onBeforeClick();
    const audioArticles = await dataAgent.getAudioArticles();
    setCurrentArticles(audioArticles);
    setLoading(false);
  };

  const handleAddFeed = async (feedUrl: string, ty: string, title: string) => {
    const res = await dataAgent.addChannel(feedUrl, ty, title)
    if (res > 0) {
      getList();
    }
  };

  const handleDeleteFeed = async (channel: ChannelType) => {
    if (channel && channel.link) {
      await dataAgent.deleteSubscription(channel.link);
      getList();
    }
  };

  // currentChannel and it's article list
  const [syncing, setSyncing] = useState(false);
  const handleRefresh = async () => {
    setSyncing(true);
    if (currentChannel) {
      // console.log("refresh current channel: ", currentChannel)
      await dataAgent.addChannel(currentChannel.link, currentChannel.ty, currentChannel.title);
      await loadArticleList(currentChannel.link);
    }
    setSyncing(false);
  };

  const updateAllReadStatus = async (feedLink: string, status: number) => {
    const res = await dataAgent.updateAllReadStatus(feedLink, status);
    if (res === 0) return;
    getList();
    await handleRefresh();
  };

  const onClickArticle = async (article: ArticleType) => {
    setCurrentArticle(article);
    store.getState().setCurrentArticle(article);
    // if (article.read_status !== 0) return;
    // update read_status to db
    const res = await dataAgent.updateArticleReadStatus(article.feed_url);
    if (res === 0) return;
    getList();
  };

  const updateStarStatus = async (url: string, status: number) => {
    await dataAgent.updateArticleStarStatus(url, status);
  };

  return (
    <ErrorBoundary>
      <Flex direction="row" h="100vh" overflow="auto" m={2}>
        <Split className="split" sizes={[20, 80]} minSize={50}>
          <Box minW={64} h="full" p={1} overflowY="auto">
            <AudioPlayer currentPod={currentPod} />
            <ChannelList 
              channelList={channelList} 
              refreshList={refreshList} 
              onShowManager={onShowManager} 
              onClickFeed={onClickFeed}
              onClickStar={onClickStar} 
              onClickAudio={onClickAudio}
              refreshing={refreshing}
              doneNum={doneNum}
            />
          </Box>
          {showManager ? (
            <Box w="full" overflowY="auto" m={1} p={2}>
              <FeedManager 
                channelList={channelList} 
                handleAddFeed={handleAddFeed}
                handleDelete={handleDeleteFeed}
              />
            </Box>
          ) : (
            <Split className="split" sizes={[25, 75]} minSize={50}>
              <Box overflowY="auto" minW={72}>
                <Channel 
                  channel={currentChannel} 
                  starChannel={starChannel} 
                  articles={currentArticles}
                  handleRefresh={handleRefresh}
                  updateAllReadStatus={updateAllReadStatus}
                  onClickArticle={onClickArticle}
                  loading={loading}
                  syncing={syncing}
                />
              </Box>
              <Box flex={1} overflowY="auto">
                <ArticleView 
                  article={currentArticle || storeArticle} 
                  starArticle={updateStarStatus}
                />
              </Box>
            </Split>
          )}
        </Split>
      </Flex>
    </ErrorBoundary>
  );
}
