import { memo } from 'react';
import { IconArrowBarToUp } from '@tabler/icons';
import { useStore } from '../../lib/store';
import { Sort } from '../../lib/userSettings';
import Tooltip from '../misc/Tooltip';
import SidebarNotesSortDropdown from './SidebarNotesSortDropdown';

type Props = {
  noteSort: Sort;
  numOfNotes: number;
};

function SidebarNotesBar(props: Props) {
  const { noteSort, numOfNotes} = props;
  const setNoteSort = useStore((state) => state.setNoteSort);

  const initDir = useStore((state) => state.initDir);
  const currentDir = useStore((state) => state.currentDir);
  const checkInit: boolean = !currentDir || currentDir === initDir; 

  const currentFolder = 'md';
  const barClass = `px-2 text-sm bg-blue-500 text-white rounded overflow-hidden`; 
  
  return (
    <div className="flex items-center justify-between border-t dark:border-gray-700">
      <div className="flex mx-2 my-1">
        <SidebarNotesSortDropdown
          currentSort={noteSort}
          setCurrentSort={setNoteSort}
        />
      </div>
      <Tooltip content={currentDir ? currentDir : 'md'}>
        <div className="flex mx-2 my-1">
          <div className="relative">
            <span className={barClass}>
              {currentFolder}: {numOfNotes}
            </span>
          </div>
        </div>
      </Tooltip>
      <Tooltip content={checkInit ? "" : "Go Upper Folder"}>
        <button
          className="p-1 mx-2 rounded hover:bg-gray-200 dark:hover:bg-gray-700"
          onClick={async (e) => {
            e.preventDefault();
            if (!currentDir || currentDir === initDir) return;
            // const parentDir: string = await getParentDir(currentDir);
            // await listDirPath(parentDir, false);
          }}
          disabled={checkInit}
        >
          <IconArrowBarToUp size={16} className="text-gray-600 dark:text-gray-300" />
        </button>
      </Tooltip>
    </div>
  );
}

export default memo(SidebarNotesBar);
