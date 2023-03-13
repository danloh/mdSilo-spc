import { useEffect, useState } from "react";
import { Box, Button, Flex, HStack, Input, Radio, RadioGroup, Stack, Text, Tooltip } from "@chakra-ui/react";
import { IconHeadphones, IconPlus, IconRss, IconTrash } from "@tabler/icons-react";
import { ChannelType } from "./types";
import * as dataAgent from "../dataAgent";

type Props = {
  channelList: ChannelType[];
  handleAddFeed: (url: string, ty: string, title: string) => Promise<void>;
  handleDelete: (channel: ChannelType) => Promise<void>;
  handleImport?: () => void;
  handleExport?: () => void;
};

export function FeedManager(props: Props) {
  const { channelList, handleAddFeed, handleDelete } = props;

  const [realList, setRealList] = useState<ChannelType[]>(channelList);
  const [showAdd, setShowAdd] = useState(false);
  const [searchText, setSearchText] = useState<string>("");

  const [feedUrl, setFeedUrl] = useState("https://www.propublica.org/feeds/propublica/main");
  const [feedType, setFeedType] = useState("rss");
  const [feedTitle, setFeedTitle] = useState("");
  const [description, setDescription] = useState("");
  const [loading, setLoading] = useState(false);
  const [confirming, setConfirming] = useState(false);

  useEffect(() => {
   setRealList(channelList);
  }, [channelList]);

  const handleLoad = async () => {
    setLoading(true);
    const res = await dataAgent.fetchFeed(feedUrl);
    // console.log("res from rust", res);
    if (!res) {
      setDescription('Cant find any feed, please check url');
      return;
    }
    const { channel } = res;
    setFeedTitle(channel.title);
    setDescription(channel.description || '');
    setLoading(false);
  };

  const handleCancel = () => {
    setLoading(false);
    setConfirming(false);
    setFeedTitle("");
    setFeedUrl("");
    setDescription("");
    setShowAdd(false);
  };

  const handleSave = async () => {
    await handleAddFeed(feedUrl, feedType, feedTitle);
    setConfirming(false);
    setShowAdd(false);
  };

  const handleSearch = (txt: string) => {
    if (txt) {
      const result = channelList.filter((item) => {
        return item.title.indexOf(txt) > -1 || item.link.indexOf(txt) > -1
      })
      setRealList(result);
    } else {
      setRealList(channelList);
    }
  };

  return (
    <Flex direction="column" p={2} w="100%" className="items-start justify-center">
      {showAdd && (
        <Flex direction="column" className="w-full">
          <HStack m={2}>
            <Text mr={2} textColor={""}>URL</Text>
            <Input
              type="text" 
              mx={2}
              placeholder="Feed URL"
              value={feedUrl}
              onChange={(e) => setFeedUrl(e.target.value)}
              autoFocus
            />
          </HStack>
          <HStack m={2}>
            <Text mr={2} textColor={""}>Title</Text>
            <Input
              type="text"
              mx={2}
              placeholder="Feed Title"
              value={feedTitle}
              onChange={(e) => setFeedTitle(e.target.value)}
              autoFocus
            />
          </HStack>
          <HStack m={2}>
            <Text mr={2} textColor={""}>Type</Text>
            <RadioGroup onChange={setFeedType} value={feedType}>
              <Stack direction='row'>
                <Radio value='rss'>RSS</Radio>
                <Radio value='atom'>Atom</Radio>
                <Radio value='podcast'>Podcast</Radio>
              </Stack>
            </RadioGroup>
          </HStack>
          <Text className="w-full m-1 dark:text-white">{description}</Text>
          <HStack my={1}>
            <Button mx={3} onClick={handleLoad}>{loading ? 'Loading...' : 'Load'}</Button>
            <Button mx={3} onClick={handleCancel}>Cancel</Button>
            <Button mx={3} onClick={handleSave}>{confirming ? 'Saving..' : 'OK'}</Button>
          </HStack>
        </Flex>
      )}
      <Flex direction="row" justifyContent="between" w="full" m={2} className="items-center">
        <Tooltip label="Add Feed" placement="bottom">
          <button
            className="px-2 py-1 text-sm text-black rounded bg-primary-200 hover:bg-primary-100"
            onClick={() => setShowAdd(!showAdd)}
          >
            <IconPlus size={15} className="" />
          </button>
        </Tooltip>
        <Box ml={4} className="">
          <input
            type="text"
            className="p-2 m-2 bg-white border-gray-200 rounded dark:bg-gray-700 dark:border-gray-700"
            placeholder="Search Feed"
            value={searchText}
            onChange={(e) => {
              setSearchText(e.target.value);
              handleSearch(e.target.value);
            }}
            onKeyDown={(e) => {
              if (e.key === 'Enter') {
                e.preventDefault();
                handleSearch(searchText);
              }
            }}
          />
        </Box>
      </Flex>
      <Flex className="w-full flex flex-col items-between justify-center border-t-2 border-gray-500 my-4">
        {realList.map((channel: ChannelType, idx: number) => {
          return (
            <HStack key={idx} className="flex items-center justify-between m-1">
              <Flex className="flex items-center justify-between">
                {channel.ty === 'rss' 
                  ? <IconRss size={12} className="mr-1 text-orange-500" /> 
                  : <IconHeadphones size={12} className="mr-1 text-purple-500" />
                }
                <span className="text-sm dark:text-white">{channel.title}</span>
              </Flex>
              <Text className="text-sm dark:text-white">{channel.link}</Text>
              <button className="cursor-pointer" onClick={async () => await handleDelete(channel)}>
                <IconTrash size={18} className="m-1 dark:text-white" />
              </button>
            </HStack>
          )
        })}
      </Flex>
    </Flex>
  );
}

// TODO: import/export OPML, EDIT FEED