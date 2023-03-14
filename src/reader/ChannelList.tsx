import { useState } from "react";
import { Box, Flex, HStack, Spinner, Stack, Image, Text, Tooltip } from "@chakra-ui/react";
import { TbHeadphones, TbPlaylist, TbRefresh, TbRss, TbSettings, TbStar } from "react-icons/tb";
import { getFavicon } from "../utils";
import { ChannelType } from "./types";

type Props = {
  channelList: ChannelType[];
  refreshList: () => Promise<void>;
  onShowManager: () => void;
  refreshing: boolean;
  doneNum: number;
  onClickFeed: (link: string) => Promise<void>;
  onClickStar: () => Promise<void>;
  onClickAudio: () => Promise<void>;
};

export function ChannelList(props: Props) {
  const { 
    channelList, refreshList, onShowManager, onClickFeed, 
    onClickStar, onClickAudio, refreshing, doneNum 
  } = props;

  const [highlighted, setHighlighted] = useState<ChannelType>();
  
  const renderFeedList = (): JSX.Element => {
    return (
      <>
        {channelList.map((channel: ChannelType, idx: number) => {
          const { unread = 0, title, ty, link } = channel;
          const ico = getFavicon(link);
          const activeClass = `${highlighted?.link === link ? 'border-l-2 border-green-500' : ''}`;
          
          return (
            <HStack 
              key={`${title}-${idx}`}
              cursor="pointer"
              className={`${activeClass}`}
              onClick={() => {
                onClickFeed(link);
                setHighlighted(channel);
              }}
            >
              <Tooltip label={channel.link} placement="top">
                <HStack mr={1}>
                  <Image src={ico} boxSize={4} mx={1} alt=">" />
                  <Text className="text-sm text-black dark:text-white">{title}</Text>
                  {ty === 'podcast' 
                    ? <TbHeadphones size={12} color="purple" />
                    : <TbRss size={12} color="orange" /> 
                  }
                </HStack>
              </Tooltip>
              <HStack>
                {/* <Text className="text-sm dark:text-white">{unread}</Text> */}
              </HStack>
            </HStack>
          );
        })}
      </>
    );
  };

  return (
    <Flex direction="column">
      <HStack spacing={2}>
        <Tooltip label="Refresh All" placement="bottom">
          <button onClick={refreshList}>
            <TbRefresh size={24} className="m-1 dark:text-white" />
          </button>
        </Tooltip>
        <Tooltip label="Manage Channel" placement="bottom">
          <button className="cursor-pointer" onClick={onShowManager}>
            <TbSettings size={24} className="m-1 dark:text-white" />
          </button>
        </Tooltip>
        {refreshing && (
          <Flex className="flex flex-col items-center justify-center">
            <Spinner className="w-4 h-4" />
            <Text className="dark:text-white">{doneNum}/{channelList.length}</Text>
          </Flex>
        )}
      </HStack>
      <Stack p={1} mt={2} className="p-1">
        <HStack direction="row" cursor="pointer" onClick={onClickStar}>
          <TbStar size={18} color="red" fill="red" />
          <Text className="m-1 dark:text-white">Starred</Text>
        </HStack>
        <HStack direction="row" cursor="pointer" onClick={onClickAudio}>
          <TbPlaylist size={18} color="purple" fill="purple" />
          <Text className="m-1 dark:text-white">Playlist</Text>
        </HStack>
        {renderFeedList()}
      </Stack>
    </Flex>
  );
}
