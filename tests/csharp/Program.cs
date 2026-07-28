using System;
using Netki;
using Putki;
using System.Collections.Generic;

namespace csharp
{
	class Factory : BufferFactory
	{
		public Bitstream.Buffer GetBuffer(uint size)
		{
			return Bitstream.Buffer.Make(new byte[size]);
		}
		public void ReturnBuffer(Bitstream.Buffer buf)
		{
			buf.buf = null;
		}
	}

	class Entry
	{
		public int ForLane;
		public byte[] Packet;
		public uint Pos, Size;
		public int ArrivalDelay;
		public int Id;
	};

	class MainClass
	{
		public static void Main(string[] args)
		{
			Random rand = new Random(30);
			PacketLane.Lane[] lanes = new PacketLane.Lane[2];
			PacketLane.LaneSetup setup = new PacketLane.LaneSetup();
			setup.Factory = new Factory();
			setup.MaxPacketSize = 512;
			setup.MinResendTimeMs = 100;
			setup.ReservedHeaderBytes = 0;
			lanes[0] = new PacketLane.Lane(0, 800);
			lanes[1] = new PacketLane.Lane(1, 800);

			List<Entry> queue = new List<Entry>();

			Dictionary<string, bool> sentMsgs = new Dictionary<string, bool>();

			int time = 0;
			int timedelta = 0;
			int sends = 0;
			int idCounter = 0;

			int iterations = 10000;
			for (int i=0;i<(iterations + 5000);i++)
			{
				timedelta = 100;
				time += timedelta;

				DateTime now = DateTime.Now.AddMilliseconds(time);
				int lane = i%2;

				uint toSend = (uint)(rand.Next() % 10);

				if (i > iterations)
				{
					Console.WriteLine("Lane "+ lane + " has " + lanes[lane].OutCount + " outgoing");
					toSend = 0;
				}
				PacketLane.ToSend[] ts = new PacketLane.ToSend[10];
				for (uint k=0;k<toSend;k++)
				{
					Bitstream.Buffer tmp = setup.Factory.GetBuffer(1024);
					if (rand.Next() % 100 < 80)
					{
						string msg = "THIS IS MESSAGE " + (sends++);
						Bitstream.PutString(tmp, msg);
						Bitstream.SyncByte(tmp);
						sentMsgs.Add(msg, false);
						tmp.Flip();
						ts[k].CanKeepData = false;
						ts[k].Data = tmp;
						ts[k].Lane = lanes[lane];
						ts[k].Reliable = true;
					}
					else
					{
						string msg = "THIS IS UNRELIABLE MESSAGE";
						Bitstream.PutString(tmp, msg);
						Bitstream.SyncByte(tmp);
						tmp.Flip();
						ts[k].CanKeepData = false;
						ts[k].Data = tmp;
						ts[k].Lane = lanes[lane];
						ts[k].Reliable = false;	
					}
				}
				PacketLane.ScheduleSend(setup, DateTime.Now.AddMilliseconds(time), ts, toSend);
				Console.WriteLine("Lane " + lane + " scheduled " + toSend + " new packets");

				uint incCount = 0;
				PacketLane.Incoming[] inc = new PacketLane.Incoming[64];
				for (int j=0;j<queue.Count;j++)
				{
					if (queue[j].ForLane != lane)
						continue;
					queue[j].ArrivalDelay -= timedelta;
					if (rand.Next()%100 < 10)
					{
						queue.RemoveAt(j--);
						continue;
					}
					if (queue[j].ArrivalDelay <= 0)
					{
						inc[incCount].Lane = lanes[lane];
						inc[incCount].ArrivalTime = DateTime.Now.AddMilliseconds(time);
						inc[incCount].CanKeepData = false;
						Bitstream.Buffer tmp = setup.Factory.GetBuffer(queue[j].Size);
						tmp.bytepos = queue[j].Pos;
						Console.WriteLine(lane + " <= " + (queue[j].Size - queue[j].Pos) + " bytes (#" + queue[j].Id + ")");

						Array.Copy(queue[j].Packet, tmp.buf, queue[j].Size);
						inc[incCount].Packet = tmp;
						incCount++;
						queue.RemoveAt(j--);
					}
				}

				if (incCount > 0)
					Console.WriteLine("Lane " + lane + " received " + incCount + " new packets");
				PacketLane.HandleIncomingPackets(setup, inc, incCount);

				uint count;
				PacketLane.Done[] done = new PacketLane.Done[128];
				PacketLane.ProcessLanesIncoming(setup, new PacketLane.Lane[1] { lanes[lane] }, done, out count);
				for (uint k=0;k<count;k++)
				{
					if (done[k].Reliable)
					{
						string msg = Bitstream.ReadString(done[k].Data);
						if (done[k].Data.error != 0)
						{
							Console.WriteLine("ERROR!");
						}
						if (sentMsgs.ContainsKey(msg) && !sentMsgs[msg])
						{
							sentMsgs[msg] = true;
							Console.WriteLine("Received message " + msg);
						}
						else
						{
							Console.WriteLine("ERROR: received unsent or duplicate of msg " + msg);
						}
					}
				}
				PacketLane.Send[] output = new PacketLane.Send[128];
				uint numOut;
				PacketLane.ProcessLanesSend(setup, new PacketLane.Lane[1] { lanes[lane] }, DateTime.Now.AddMilliseconds(time), output, out numOut);
				for (uint k=0;k<numOut;k++)
				{
					Entry e = new Entry();
					e.ArrivalDelay = rand.Next() % 1000 + 10;
					e.ForLane = 1-lane;
					e.Packet = output[k].Data.buf;
					e.Pos = output[k].Data.bytepos;
					e.Size = output[k].Data.bytesize;
					e.Id = idCounter++;
					queue.Add(e);
					Console.WriteLine(lane + " => " + (e.Size - e.Pos) + " bytes (#" + e.Id + ")");
				}
			}

			foreach (var key in sentMsgs.Keys)
			{
				if (!sentMsgs[key])
				{
					Console.WriteLine("Did not ever receive message " + key);
				}
			}

			for (int w=0;w<2;w++)
			{
				int l0 = w;
				int l1 = 1-w;

				if (lanes[l0].OutCount > 0)
				{
					bool found = false;		
					uint lowest = 10000000;
					for (int i=0;i<lanes[l0].OutCount;i++)
					{
						if (lanes[l0].Out[i].Source == null)
							continue;
						if (lanes[l0].Out[i].SeqId < lowest)
							lowest = lanes[l0].Out[i].SeqId;
						if (lanes[l0].Out[i].SeqId == (lanes[l1].ReliableSeq + 1))
						{
							found = true;
						}
					}
					if (!found)
					{
						Console.WriteLine("ERROR!! " + l1 + " waiting for " + (lanes[l1].ReliableSeq+1) + " but lowest scheduled in " + l0 + " is " + lowest);
					}
					Console.WriteLine("lane " + l0 + " had pending packet for " + l1 + "=" + found);
				}
			}

		}
	}
}
