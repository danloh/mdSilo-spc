/* eslint-disable @typescript-eslint/no-explicit-any */
import { useState, useCallback, useMemo, useEffect } from 'react';
import MsEditor from 'mdsmirror';
import Board from 'react-trello';
import { useStore } from '../../lib/store';
import ErrorBoundary from '../misc/ErrorBoundary';
import { matchSort, SearchLeaf } from '../sidebar/SidebarSearch';
import Tree, { TreeNode } from '../misc/Tree';
import useNoteSearch from '../../editor/hooks/useNoteSearch';
import useTasks from '../../editor/hooks/useTasks';
import useOnNoteLinkClick from '../../editor/hooks/useOnNoteLinkClick';
import { getStrDate } from '../../utils/helper';

export default function Tasks() {
  const isLoaded = useStore((state) => state.isLoaded);
  const setIsLoaded = useStore((state) => state.setIsLoaded);
  const initDir = useStore((state) => state.initDir);

  const [tab, setTab] = useState<string>("kanban");
  const [kanbanData, setKanbanData] = useState<any>({lanes: []});
  const kanbanJsonPath = '';
  // console.log("t loaded?", isLoaded);
  useEffect(() => {
    // if (!isLoaded && initDir) {
    //   loadDir(initDir).then(() => setIsLoaded(true));
    // }
    // if (kanbanJsonPath) {
    //   //
    //   const jsonFile = new FileAPI(kanbanJsonPath);
    //   jsonFile.readJSONFile().then(json => setKanbanData(json));
    // }
  }, [initDir, isLoaded, kanbanJsonPath, setIsLoaded]);

  const darkMode = useStore((state) => state.darkMode);
  const isRTL = useStore((state) => state.isRTL);
  const { onClick: onNoteLinkClick } = useOnNoteLinkClick();

  const onKanbanChange = useCallback(
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    async (newData: any) => {
      // const saveFile = new FileAPI('kanban.json', initDir);
      // await saveFile.writeFile(JSON.stringify(newData));
      setKanbanData(newData);
    },
    [initDir]
  );

  // per checkbox
  // 
  const checkboxTasks = useTasks();
  const allTasks = checkboxTasks
    .map(task => task.tasks)
    .flat()
    .sort((a, b) => Number(a.completed) - Number(b.completed));
  const [perNote, setPerNote] = useState(true);

  // per hashtag
  // 
  const [collapseIds, setCollapseIds] = useState<string[]>([]);
  const onClose = useCallback(
    (ids: string[]) => setCollapseIds(ids), []
  );

  const search = useNoteSearch({ searchHashTag: true, extendedSearch: true });
  const getTaskNotes = useCallback((searchQuery: string) => {
    const searchResults = search(searchQuery);
    // console.log("search res: ", searchQuery, searchResults);
    return searchResults.map((result, index) => ({
      id: `${result.item.id}-${index}`,
      labelNode: (
        <p className="py-1 mt-2 text-sm overflow-hidden overflow-ellipsis whitespace-nowrap dark:text-gray-200">
          {`${getStrDate(result.item.update_at)} : ${result.item.title}`}
        </p>
      ),
      showArrow: false,
      children: result.matches
        ? [...result.matches].sort(matchSort).map((match, index) => ({
            id: `${result.item.id}-${index}`,
            labelNode: (
              <SearchLeaf
                noteId={result.item.id}
                text={match.value ?? ''}
                searchQuery={searchQuery}
                block={
                  result.item.blocks && match.refIndex !== undefined
                    ? result.item.blocks[match.refIndex]
                    : undefined
                }
                className={getTaskClass(searchQuery)}
              />
            ),
            showArrow: false,
            toIndent: false,
          }))
        : undefined,
    }))
  }, [search]);

  const taskDivClass = 'flex mt-1 p-1 rounded';
  const tasks: TreeNode[] = useMemo(() => [
    {
      id: 'doing',
      labelNode: (
        <div className={taskDivClass}>
          <b className="py-1 text-xl">Doing</b>
        </div>
      ),
      children: getTaskNotes('doing'),
    },
    {
      id: 'todo',
      labelNode: (
        <div className={taskDivClass}>
          <b className="py-1 text-xl">To Do</b>
        </div>
      ),
      children: getTaskNotes('todo'),
    },
    {
      id: 'done',
      labelNode: (
        <div className={taskDivClass}>
          <b className="py-1 text-xl">Done</b>
        </div>
      ),
      children: getTaskNotes('done'),
    },
  ], [getTaskNotes]);

  return (
    <ErrorBoundary>
      <div className="h-full">
        <div className="flex p-1 rounded">
          <button 
            className={`text-md mx-2 ${tab === "kanban" ? 'text-red-500' : 'text-blue-500'}`} 
            onClick={() => setTab("kanban")}
          >
            Kanban
          </button>  
          <button 
            className={`text-md mx-2 ${tab != "kanban" ? 'text-red-500' : 'text-blue-500'}`}
            onClick={() => setTab("list")}
          >
            List
          </button>
        </div>
        {tab === "kanban" ? (
        <div className="p-1">
          <Board
            // style={{backgroundColor: "rgb(138, 146, 153)"}}
            data={kanbanData}
            draggable
            editable
            canAddLanes
            editLaneTitle 
            collapsibleLanes
            id="kanban"
            onDataChange={onKanbanChange}
            onCardDelete={() => {/**/}}
            onCardAdd={() => {/**/}}
          />
        </div>
        ) : (
        <div className="flex flex-1 flex-col flex-shrink-0 md:flex-shrink p-6 w-full mx-auto md:w-128 lg:w-160 xl:w-192 bg-white dark:bg-black dark:text-gray-200 overlfow-y-auto">
          <div className="flex my-1 p-1 rounded">
            <button 
              className={`text-xl mr-2 ${perNote ? 'text-red-500' : ''}`} 
              onClick={() => setPerNote(true)}
            >
              PER NOTE
            </button>  
            <button 
              className={`text-xl ml-2 ${perNote ? '' : 'text-red-500'}`}
              onClick={() => setPerNote(false)}
            >
              PER COMPLETION
            </button>
          </div>
          <div className="my-2">
            {perNote ? (
              <>
              {checkboxTasks.map((doc, idx) => (
                <div key={idx}> 
                  <button 
                    className="block text-left text-xl rounded p-2 my-1 w-full break-words"
                    onClick={() => onNoteLinkClick(doc.note.id)}
                  >
                    {`${getStrDate(doc.note.updated_at)} : ${doc.note.title}`}
                  </button>
                  <MsEditor 
                    value={doc.tasks.reduce(
                      (box, item) => box + '\n' + `- [${item.completed ? 'X' : ' '}] ${item.text}`, '')
                    } 
                    readOnly={true} 
                    dark={darkMode} 
                    dir={isRTL ? 'rtl' : 'ltr'} 
                  />
                </div>
              ))}
              </>
            ) : (
              <div> 
                <MsEditor 
                  value={allTasks.reduce(
                    (box, item) => box + '\n' + `- [${item.completed ? 'X' : ' '}] ${item.text}`, '')
                  } 
                  readOnly={true} 
                  dark={darkMode} 
                  dir={isRTL ? 'rtl' : 'ltr'}
                />
              </div>
            )}
          </div>
          <div className="flex my-1 p-1 rounded">
            <b className="text-xl mr-2">PER HASHTAG: </b>
            <button className="text-red-500 text-xl mr-2" onClick={() => onClose(['done','todo'])}>
              #doing
            </button>  
            <button className="text-blue-500 text-xl mr-2" onClick={() => onClose(['done','doing'])}>
              #todo
            </button>
            <button className="text-green-500 text-xl" onClick={() => onClose(['doing','todo'])}>
              #done
            </button>
          </div>
          <div className="overlfow-y-auto">
            <Tree data={tasks} className={""} collapseAll={false} collapseIds={collapseIds} />
          </div>
        </div>)}
      </div>
    </ErrorBoundary>
  );
}

const getTaskClass = (taskTag: string) => {
  const tagClass = 'link bg-gray-200 dark:bg-gray-700 px-2 border-l-2';
  switch (taskTag) {
    case '#todo':
      return `${tagClass} border-blue-600`;
    case '#done':
      return `${tagClass} border-green-600`; 
    case '#doing':
      return `${tagClass} border-red-600`;
    default:
      return tagClass;
  }
}
