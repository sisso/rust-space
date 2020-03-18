// <auto-generated>
//  automatically generated by the FlatBuffers compiler, do not modify
// </auto-generated>

namespace space_data
{

using global::System;
using global::FlatBuffers;

public struct EntityNew : IFlatbufferObject
{
  private Struct __p;
  public ByteBuffer ByteBuffer { get { return __p.bb; } }
  public void __init(int _i, ByteBuffer _bb) { __p = new Struct(_i, _bb); }
  public EntityNew __assign(int _i, ByteBuffer _bb) { __init(_i, _bb); return this; }

  public uint Id { get { return __p.bb.GetUint(__p.bb_pos + 0); } }
  public space_data.V2 Pos { get { return (new space_data.V2()).__assign(__p.bb_pos + 4, __p.bb); } }
  public uint SectorId { get { return __p.bb.GetUint(__p.bb_pos + 12); } }
  public space_data.EntityKind Kind { get { return (space_data.EntityKind)__p.bb.GetShort(__p.bb_pos + 16); } }

  public static Offset<space_data.EntityNew> CreateEntityNew(FlatBufferBuilder builder, uint Id, float pos_X, float pos_Y, uint SectorId, space_data.EntityKind Kind) {
    builder.Prep(4, 20);
    builder.Pad(2);
    builder.PutShort((short)Kind);
    builder.PutUint(SectorId);
    builder.Prep(4, 8);
    builder.PutFloat(pos_Y);
    builder.PutFloat(pos_X);
    builder.PutUint(Id);
    return new Offset<space_data.EntityNew>(builder.Offset);
  }
};


}
