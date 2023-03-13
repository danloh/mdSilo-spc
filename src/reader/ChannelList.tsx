import { useState } from "react";
import { Box, Flex, HStack, Spinner, Stack, Image, Text, Tooltip } from "@chakra-ui/react";
import { IconHeadphones, IconRefresh, IconRss, IconSettings, IconStar } from "@tabler/icons-react";
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
};

export function ChannelList(props: Props) {
  const { channelList, refreshList, onShowManager, onClickFeed, onClickStar, refreshing, doneNum } = props;

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
                <HStack className="flex flex-row items-center justify-start mr-1">
                  <Image src={ico} boxSize={4} mx={1} alt=">" />
                  <Text className="text-sm text-black dark:text-white">{title}</Text>
                  {ty === 'podcast' 
                    ? <IconHeadphones size={12} color="purple" />
                    : <IconRss size={12} color="orange" /> 
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
    <Flex direction="column" className="flex flex-col">
      <HStack spacing={2}>
        <Tooltip label="Refresh All" placement="bottom">
          <button className="cursor-pointer" onClick={refreshList}>
            <IconRefresh size={24} className="m-1 dark:text-white" />
          </button>
        </Tooltip>
        <Tooltip label="Manage Channel" placement="bottom">
          <button className="cursor-pointer" onClick={onShowManager}>
            <IconSettings size={24} className="m-1 dark:text-white" />
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
        <HStack 
          direction="row"
          cursor="pointer"
          onClick={onClickStar}
        >
          <IconStar size={18} className="m-1 fill-yellow-500 text-yellow-500" />
          <Text className="m-1 dark:text-white">Starred</Text>
        </HStack>
        {renderFeedList()}
      </Stack>
    </Flex>
  );
}
