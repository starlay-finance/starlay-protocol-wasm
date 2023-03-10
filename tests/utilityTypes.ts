import { ConditionalExcept, ConditionalPick, StringKeyOf } from 'type-fest'

export type ReplacedType<BaseType, From, To> = ConditionalExcept<
  BaseType,
  From
> & {
  [key in StringKeyOf<ConditionalPick<BaseType, From>>]: To
}
