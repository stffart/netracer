import { Table, flexRender, Column } from '@tanstack/react-table';
import { useState, useEffect } from 'react';
import DateTimePicker from 'react-datetime-picker';
import useTable from './useTable';

interface IProps<T> {
  isSortable?: boolean;
}

function Filter({ column }: { column: Column<any, unknown> }) {
  const columnFilterValue = column.getFilterValue()
  const { filterVariant } = column.columnDef.meta ?? {} 

  return filterVariant === 'datetime' ? (
    <div className="datepickerWrapper">
      <DateTimePicker onChange={value => column.setFilterValue(value)}
         value={columnFilterValue as Date} 
       />
    </div>
  ) : filterVariant === 'range' ? (
    <div>
      <div className="flex space-x-2">
        {/* See faceted column filters example for min max values functionality */}
        <DebouncedInput
          type="number"
          min={Number(column.getFacetedMinMaxValues()?.[0] ?? '')}
          max={Number(column.getFacetedMinMaxValues()?.[1] ?? '')}
          value={(columnFilterValue as [number, number])?.[0] ?? ''}
          onChange={value =>
            column.setFilterValue((old: [number, number]) => { 
                 if ( old?.[1] ) {
                    return [value, old?.[1]];
                 } else {
                    return [value, value];
                 }
              }
            )
          }
          placeholder={`Min`}
          className="w-24 border shadow rounded taninput"
        />
        <DebouncedInput
          type="number"
          value={(columnFilterValue as [number, number])?.[1] ?? ''}
          onChange={value =>
            column.setFilterValue((old: [number, number]) => {
                    return [old?.[0], value];
              }
            )
          }
          placeholder={`Max`}
          className="w-24 border shadow rounded taninput"
        />
      </div>
    </div>
  ) : filterVariant === 'select' ? (
    <select
      onChange={e => column.setFilterValue(e.target.value)}
      value={columnFilterValue?.toString()}
      className="protocolselect"
    >
      {/* See faceted column filters example for dynamic select options */}
      <option value="">ALL</option>
      <option value="TCP">TCP</option>
      <option value="UDP">UDP</option>
    </select>
  ) : (
    <DebouncedInput
      className="w-36 border shadow rounded taninput"
      onChange={value => column.setFilterValue(value)}
      placeholder={`Search...`}
      type="text"
      value={(columnFilterValue ?? '') as string}
    />
    // See faceted column filters example for datalist search suggestions
  )
}

// A typical debounced input react component
function DebouncedInput({
  value: initialValue,
  onChange,
  debounce = 500,
  ...props
}: {
  value: string | number
  onChange: (value: string | number) => void
  debounce?: number
} & Omit<React.InputHTMLAttributes<HTMLInputElement>, 'onChange'>) {
  const [value, setValue] = useState(initialValue)

  useEffect(() => {
    setValue(initialValue)
  }, [initialValue])

  useEffect(() => {
    const timeout = setTimeout(() => {
      onChange(value)
    }, debounce)

    return () => clearTimeout(timeout)
  }, [value])

  return (
    <input {...props} value={value} onChange={e => setValue(e.target.value)} />
  )
}

const THead = <T,>({ isSortable }: IProps<T>) => {
  const table = useTable();

  if (!table) return null;
  return (
    <thead className="text-xs text-gray-700 uppercase bg-gray-50 dark:bg-gray-700 dark:text-gray-400">
      <tr>
        {table.getHeaderGroups().map((x) => {
          return x.headers.map((header) => {
            return (
              <th key={header.id} scope="col" className="px-6 py-3">
                {header.isPlaceholder ? null : (
                  <div className="flex">
                    <div className="flex flex-1 cursor-pointer headerdiv" onClick={isSortable ? header.column.getToggleSortingHandler() : undefined}>
                      {flexRender(header.column.columnDef.header, header.getContext())}
                      {{
                        asc: <i className="fa-solid fa-sort-up ml-1"></i>,
                        desc: <i className="fa-solid fa-sort-down ml-1"></i>,
                      }[header.column.getIsSorted() as string] ?? null}
                    </div>
                    {header.column.getCanFilter() ? (
                          <div>
                            <Filter column={header.column} />
                          </div>
                        ) : null}
                  </div>
                )}
              </th>
            );
          });
        })}
      </tr>
    </thead>
  );
};

export default THead;
