import { useQuery } from '@tanstack/react-query';
import { createColumnHelper, 
         getCoreRowModel, 
         getPaginationRowModel, 
         getFilteredRowModel, 
         getSortedRowModel,
         getFacetedMinMaxValues, 
         FilterFn,
         RowData,
         Row,
         useReactTable } from '@tanstack/react-table';
import NavBar from 'components/NavBar';
import Page, { PageBits } from 'components/Page';
import Connections from 'modules/netracer';
import { Connection } from 'modules/netracer/types';
import TanstackTable from 'components/TanstackTable';
import { useState, useRef, useEffect, ReactNode, ReactElement } from 'react';
import 'react-datetime-picker/dist/DateTimePicker.css';
import 'react-calendar/dist/Calendar.css';
import 'react-clock/dist/Clock.css';

const columnHelper = createColumnHelper<Connection>();

declare module '@tanstack/react-table' {
  //allows us to define custom properties for our columns
  interface ColumnMeta<TData extends RowData, TValue> {
    filterVariant?: 'text' | 'range' | 'select' | 'datetime'
  }
}


const myDateFilterFn: FilterFn<Connection> = (row: Row<Connection>, columnId: string, filterValue: any, addMeta: (meta: any) => void) => {
  if (filterValue == null) return true;
  return parseInt(row.getValue<string>(columnId)) > filterValue.getTime()/1000;
}

const myRangeFilterFn: FilterFn<Connection> = (row: Row<Connection>, columnId: string, filterValue: any, addMeta: (meta: any) => void) => {
  let minValue = (filterValue as [number, number])?.[0];
  if (!minValue) minValue = 0;
  let maxValue = (filterValue as [number, number])?.[1]; 
  if (!maxValue) maxValue = 65535;

  const vals = row.getValue<string>(columnId).split(',');
  for (const v of vals) {
     const intval = parseInt(v.replace(" ",""));
     const inRange = (intval >= minValue) && (intval <= maxValue); 
     if (inRange) {
       return true;
     }
  }   
  return false;
}

interface Props {
    children?: ReactNode
}

const Collapse = ({children}: Props) => {
  const [open, setOpen] = useState(false);
  const [cl, setCl] = useState("tdcollapsed");
  const [height, setHeight] = useState(0);
  const [clickTime, setClickTime] = useState(0);
  const [pos, setPos] = useState({x: 0, y: 0});
  const ref: React.RefObject<HTMLInputElement> = useRef(null)
  useEffect(() => {
    let mounted = true;    
    if (open !== null && mounted) {
      if (open) setCl("tdexpanded"); else setCl("tdcollapsed");
    }
    if (ref.current != undefined)
      setHeight(ref.current.scrollHeight);

    return () => {
      mounted = false;
    };
  }, [open]);

  return (
    <div className="tdcollapse" 
      onMouseDown={ ({nativeEvent: e}) => {
                      setClickTime(Date.now()); setPos({x: e.x, y: e.y});
                    }
                  }
      onMouseUp={ ({nativeEvent: e}) => {
                      console.log(e)
                      console.log(pos)
                      if ( Date.now() - clickTime < 200 && Math.abs(pos.x - e.x) < 5 && Math.abs(pos.y - e.y) < 5 ) 
                        setOpen(state => !state);
                  }
                }
    >
      <div className={cl} ref={ref}>
      {children}
      </div>
      {!open && height > 200 && <div className="tdellipsis">...</div>}
    </div>
  );
};

const columns = [
  columnHelper.accessor('addr.src', {
    header: () => <span>Sources</span>,
    footer: (info) => info.column.id,
    cell: ({ getValue }) => (
       <span>
        {Array.isArray(getValue().split(' ')) ? (
          <Collapse>
            <ul>
            {(getValue().split(' ') as string[]).map((item: string, index: number) => (
              <div key={item}>{item}</div>
            ))}
            </ul>
          </Collapse>
        ) : (
          (getValue() as React.ReactNode)
        )}
      </span>
    ),
    meta: {
      filterVariant: 'text'
    }

  }),
//     cell: (info) => info.getValue(),

  columnHelper.accessor('addr.dst', {
    header: () => <span>Destination</span>,
    footer: (info) => info.column.id,
    cell: ({ getValue }) => (
       <span>
        {Array.isArray(getValue().split(' ')) ? (
        <Collapse>
          <ul>
            {(getValue().split(' ') as string[]).map((item: string, index: number) => (
              <div key={index}>{item}</div>
            ))}
          </ul>
        </Collapse>
        ) : (
          (getValue() as React.ReactNode)
        )}
      </span>
    ),
    meta: {
      filterVariant: 'text'
    }
    
  }),
  columnHelper.accessor('addr.port', {
    cell: (info) =>  (<Collapse>{info.getValue().toString()}</Collapse>),
    header: () => <span>Port</span>,
    footer: (info) => info.column.id,
    filterFn: myRangeFilterFn,
    meta: {
      filterVariant: 'range'
    }
  }),
  columnHelper.accessor('addr.protocol', {
    cell: (info) => info.getValue(),
    header: () => <span>Protocol</span>,
    footer: (info) => info.column.id,
    meta: {
      filterVariant: 'select'
    }
  }),
  columnHelper.accessor('time', {
    cell: (info) => new Date(info.getValue()*1000).toLocaleString(),
    header: () => <span>LastTime</span>,
    footer: (info) => info.column.id,
    filterFn: myDateFilterFn,
    meta: {
      filterVariant: 'datetime'
    }
  }),
];

const Home = () => {
  const { data } = useQuery(['/conagg'], Connections.all, { initialData: [] });
  const [pagination, setPagination] = useState({
    pageIndex: 0, //initial page index
    pageSize: 1000, //default page size
  });
  const table = useReactTable({
    data,
    columns,
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
    getPaginationRowModel: getPaginationRowModel(),
    onPaginationChange: setPagination, //update the pagination state when internal APIs mutate the pagination state
    getFilteredRowModel: getFilteredRowModel(),
    getFacetedMinMaxValues: getFacetedMinMaxValues(),
    state: {
      pagination
    }
  });

  return (
    <Page drawer>
      <NavBar />
      <PageBits.Content>
        <TanstackTable.Table table={table} >
          <TanstackTable.Caption>
            <p className="mt-1 text-sm font-normal text-gray-500 dark:text-gray-400">
            </p>
          </TanstackTable.Caption>

          <TanstackTable.THead isSortable />
          <TanstackTable.TBody />
        </TanstackTable.Table>
      </PageBits.Content>
    </Page>
  );
};

export default Home;
