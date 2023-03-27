import { ConditionalExcept, ConditionalPick, StringKeyOf } from 'type-fest'

export type ReplacedType<BaseType, From, To> = ConditionalExcept<
  BaseType,
  From
> & {
  [key in StringKeyOf<ConditionalPick<BaseType, From>>]: To
}

export type ExcludeLastArrayElement<ValueType extends unknown[]> =
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  ValueType extends [infer _]
    ? []
    : ValueType extends [infer Head, ...infer Tail]
    ? [Head, ...ExcludeLastArrayElement<Tail>]
    : []
